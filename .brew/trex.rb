class Trex < Formula
  desc "Terminal-based collaborative text editor with P2P sync and post-quantum security"
  homepage "https://github.com/cristiano-s-r-filho/telarex"
  version "0.5.1"
  url "https://github.com/cristiano-s-r-filho/telarex/archive/refs/tags/v0.5.1.tar.gz"
  sha256 "938d3a0f9ae4124502b2669144244e16d8d74857cdd4c5e231c6de16cca98a96"
  license "MIT OR Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/telarex-tui")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/trex --version")
  end
end
