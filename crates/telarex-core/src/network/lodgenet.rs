use libp2p::{
    futures::StreamExt,
    gossipsub, mdns, noise, tcp, yamux, identify, kad, ping,
    kad::store::MemoryStore,
    SwarmBuilder,
    PeerId,
};
use std::time::Duration;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{NetworkEvent, NetworkCommand};

struct PendingJoinData {
    peer_id: String,
    public_key: Vec<u8>,
    challenge: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
enum WireMessage {
    Discovery { id: Uuid, name: String, owner: String, member_count: u32 },
    Join { id: Uuid, public_key: Vec<u8>, username: String },
    Leave { id: Uuid },
    Sync { id: Uuid, path: PathBuf, data: Vec<u8> },
    Challenge { id: Uuid, challenge: Vec<u8> },
    Response { id: Uuid, proof: Vec<u8> },
    Presence { id: Uuid, username: String },
    // BOOTSTRAP: Explicitly ask for a lodge's owner on the global topic
    SeekLodge { id: Uuid },
    JoinApproved { id: Uuid, peer_id: String },
    JoinRejected { id: Uuid, peer_id: String },
}

pub struct NetworkManager {
    event_tx: mpsc::Sender<NetworkEvent>,
    cmd_rx: mpsc::Receiver<NetworkCommand>,
}

impl NetworkManager {
    pub fn new(event_tx: mpsc::Sender<NetworkEvent>, cmd_rx: mpsc::Receiver<NetworkCommand>) -> Self {
        Self { event_tx, cmd_rx }
    }

    pub async fn start(self, identity_seed: String, listen_addr: Option<String>) -> anyhow::Result<()> {
        let mut seed_bytes = [0u8; 32];
        let decoded_seed = hex::decode(&identity_seed).unwrap_or_else(|_| vec![0;32]);
        if decoded_seed.len() >= 32 {
            seed_bytes.copy_from_slice(&decoded_seed[..32]);
        }
        
        let id_keys = libp2p::identity::Keypair::ed25519_from_bytes(seed_bytes)?;
        let local_peer_id = PeerId::from(id_keys.public());

        let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(|e| anyhow::anyhow!(e))?;
                
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).map_err(|e: &str| anyhow::anyhow!(e))?;

                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
                
                let mut kad_config = kad::Config::default();
                kad_config.set_record_ttl(Some(Duration::from_secs(60 * 60 * 24))); 
                let kademlia = kad::Behaviour::with_config(local_peer_id, MemoryStore::new(local_peer_id), kad_config);
                
                let identify = identify::Behaviour::new(identify::Config::new(
                    "/telarex/1.0.0".into(),
                    key.public(),
                ));

                let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(30)));

                Ok(MyBehaviour { gossipsub, mdns, kademlia, identify, ping })
            })?
            .build();

        // GLOBAL TOPIC for discovery
        let global_topic = gossipsub::IdentTopic::new("telarex-lodgenet-discovery");
        swarm.behaviour_mut().gossipsub.subscribe(&global_topic)?;

        let addr: libp2p::Multiaddr = listen_addr.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".to_string()).parse()?;
        swarm.listen_on(addr)?;

        let tx = self.event_tx;
        let rx_tx = tx.clone();
        let mut rx = self.cmd_rx;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                if let Err(_) = rx_tx.send(NetworkEvent::Tick).await {
                    break;
                }
            }
        });

        let mut local_lodges: HashMap<Uuid, String> = HashMap::new();
        let mut lodge_members: HashMap<Uuid, Vec<String>> = HashMap::new();
        let mut pending_challenges: HashMap<Uuid, Vec<PendingJoinData>> = HashMap::new();
        let mut active_topics: HashMap<Uuid, gossipsub::IdentTopic> = HashMap::new();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => {
                        match event {
                            libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                                for (peer_id, multiaddr) in list {
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                                    let _ = tx.send(NetworkEvent::PeerConnected(peer_id.to_string())).await;
                                }
                            }
                            libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                                for addr in info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }
                            libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { 
                                result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))), 
                                .. 
                            })) => {
                                if let Ok(peer_id) = PeerId::from_bytes(&record.record.value) {
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                }
                            }
                            libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                propagation_source,
                                message,
                                ..
                            })) => {
                                if let Ok(wire_msg) = serde_json::from_slice::<WireMessage>(&message.data) {
                                    match wire_msg {
                                        WireMessage::Discovery { id, name, owner, member_count: _ } => {
                                            let display_name = format!("{} by {}", name, owner);
                                            let _ = tx.send(NetworkEvent::LodgeDiscovery { id, name: display_name, peer_id: propagation_source.to_string() }).await;
                                        }
                                        WireMessage::Join { id, public_key, username } => {
                                            if local_lodges.contains_key(&id) {
                                                let challenge = rand::random::<[u8; 32]>().to_vec();
                                                let peer_str = propagation_source.to_string();
                                                let entry = pending_challenges.entry(id).or_insert_with(Vec::new);
                                                entry.push(PendingJoinData {
                                                    peer_id: peer_str.clone(),
                                                    public_key: public_key.clone(),
                                                    challenge: challenge.clone(),
                                                });
                                                let _ = tx.send(NetworkEvent::JoinRequest {
                                                    lodge_id: id,
                                                    peer_id: peer_str,
                                                    username,
                                                    public_key,
                                                }).await;
                                                let _ = tx.send(NetworkEvent::AuthChallenge { lodge_id: id, challenge }).await;
                                            }
                                        }
                                        WireMessage::Leave { id } => {
                                            if let Some(members) = lodge_members.get_mut(&id) {
                                                members.retain(|m| m != &propagation_source.to_string());
                                                let _ = tx.send(NetworkEvent::LodgeMembersUpdated { lodge_id: id, members: members.clone() }).await;
                                            }
                                        }
                                        WireMessage::Challenge { id, challenge } => {
                                            let _ = tx.send(NetworkEvent::AuthChallenge { lodge_id: id, challenge }).await;
                                        }
                                        WireMessage::Response { id, proof } => {
                                            if let Some(entries) = pending_challenges.get_mut(&id) {
                                                if let Some(pos) = entries.iter().position(|e| e.challenge.len() > 0) {
                                                    let entry = entries.remove(pos);
                                                    let _ = tx.send(NetworkEvent::AuthVerify {
                                                        lodge_id: id,
                                                        challenge: entry.challenge,
                                                        proof,
                                                        public_key: entry.public_key,
                                                    }).await;
                                                }
                                            }
                                        }
                                        WireMessage::Sync { id, path, data } => {
                                            let _ = tx.send(NetworkEvent::SyncMessage { lodge_id: id, path, data }).await;
                                        }
                                        WireMessage::Presence { id, username } => {
                                            let members = lodge_members.entry(id).or_insert_with(Vec::new);
                                            if !members.contains(&username) {
                                                members.push(username);
                                                let _ = tx.send(NetworkEvent::LodgeMembersUpdated { lodge_id: id, members: members.clone() }).await;
                                            }
                                        }
                                        WireMessage::SeekLodge { id } => {
                                            if let Some(name) = local_lodges.get(&id) {
                                                // RE-BROADCAST discovery if someone asks
                                                let _ = tx.send(NetworkEvent::LodgeDiscovery { id, name: name.clone(), peer_id: local_peer_id.to_string() }).await;
                                            }
                                        }
                                        WireMessage::JoinApproved { id, peer_id: _ } => {
                                            let _ = tx.send(NetworkEvent::LodgeJoined { lodge_id: id }).await;
                                        }
                                        WireMessage::JoinRejected { id, peer_id: _ } => {
                                            let _ = tx.send(NetworkEvent::JoinRejected { lodge_id: id }).await;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        let mut target_topic = global_topic.clone();
                        let wire_msg = match cmd {
                            NetworkCommand::ShareLodge { id, name } => {
                                local_lodges.insert(id, name.clone());
                                let topic = gossipsub::IdentTopic::new(format!("telarex-lodge-{}", id));
                                let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
                                active_topics.insert(id, topic);
                                
                                let key = kad::RecordKey::new(id.as_bytes());
                                let record = kad::Record {
                                    key,
                                    value: local_peer_id.to_bytes(),
                                    publisher: None,
                                    expires: None,
                                };
                                let _ = swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One);
                                Some(WireMessage::Discovery { 
                                    id, 
                                    name, 
                                    owner: "me".to_string(), 
                                    member_count: 1 
                                })
                            }
                            NetworkCommand::SendSync { lodge_id, path, data } => {
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::Sync { id: lodge_id, path, data })
                            }
                            NetworkCommand::JoinLodge { lodge_id, public_key, username } => {
                                // HARDENING: Actively seek peers for this lodge
                                let _ = swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(lodge_id.as_bytes()));
                                let topic = gossipsub::IdentTopic::new(format!("telarex-lodge-{}", lodge_id));
                                let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
                                active_topics.insert(lodge_id, topic);
                                
                                // Broadcast seek request on global topic
                                if let Ok(encoded) = serde_json::to_vec(&WireMessage::SeekLodge { id: lodge_id }) {
                                    let _ = swarm.behaviour_mut().gossipsub.publish(global_topic.clone(), encoded);
                                }

                                Some(WireMessage::Join { id: lodge_id, public_key, username })
                            }
                            NetworkCommand::LeaveLodge { lodge_id } => {
                                if let Some(topic) = active_topics.remove(&lodge_id) {
                                    let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
                                }
                                let _ = tx.send(NetworkEvent::LodgeLeft { lodge_id }).await;
                                Some(WireMessage::Leave { id: lodge_id })
                            }
                            NetworkCommand::Disconnect => {
                                for topic in active_topics.values() {
                                    let _ = swarm.behaviour_mut().gossipsub.unsubscribe(topic);
                                }
                                let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&global_topic);
                                let _ = tx.send(NetworkEvent::NetworkShutdown).await;
                                None
                            }
                            NetworkCommand::AnnouncePresence { lodge_id, username } => {
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::Presence { id: lodge_id, username })
                            }
                            NetworkCommand::SendAuthChallenge { lodge_id, challenge } => {
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::Challenge { id: lodge_id, challenge })
                            }
                            NetworkCommand::SendAuthResponse { lodge_id, proof } => {
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::Response { id: lodge_id, proof })
                            }
                            NetworkCommand::ApproveJoin { lodge_id, peer_id } => {
                                let _ = swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(lodge_id.as_bytes()));
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::JoinApproved { id: lodge_id, peer_id })
                            }
                            NetworkCommand::RejectJoin { lodge_id, peer_id } => {
                                if let Some(topic) = active_topics.get(&lodge_id) {
                                    target_topic = topic.clone();
                                }
                                Some(WireMessage::JoinRejected { id: lodge_id, peer_id })
                            }
                        };

                        if let Some(msg) = wire_msg {
                            if let Ok(encoded) = serde_json::to_vec(&msg) {
                                let _ = swarm.behaviour_mut().gossipsub.publish(target_topic, encoded);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for MyBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        MyBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}

impl From<kad::Event> for MyBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        MyBehaviourEvent::Kademlia(event)
    }
}

impl From<identify::Event> for MyBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        MyBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for MyBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        MyBehaviourEvent::Ping(event)
    }
}
