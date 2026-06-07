class Trex < Formula
  desc "Terminal-based collaborative text editor with P2P sync and post-quantum security"
  homepage "https://github.com/cristiano-s-r-filho/telarex"
  url "https://github.com/cristiano-s-r-filho/telarex/archive/refs/tags/v0.5.0-beta.tar.gz"
  sha256 "9818769f171bee29b5fbc3a9608d1edf054192a068ec21edd60721c7bc675c31"
  license "MIT OR Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/telarex-tui")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/trex --version")
  end
end
