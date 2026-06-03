class PawsGames < Formula
  desc "Community game library for Paws — Dog Jump, Earth Online, Tetris"
  homepage "https://github.com/interesting-vibe-coding/paws-games"
  license "MIT"

  stable do
    url "https://github.com/interesting-vibe-coding/paws-games/archive/refs/tags/v0.3.0.tar.gz"
    sha256 "6e399fc9362bd83a4ebc5e0cf8cf0949f45a69f783027ff8557301ce524f7a6f"
    version "0.3.0"
  end

  head "https://github.com/interesting-vibe-coding/paws-games.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--bin", "jump-high", *std_cargo_args
    system "cargo", "install", "--bin", "earth-online", *std_cargo_args
    system "cargo", "install", "--bin", "tetris", *std_cargo_args
  end

  test do
    assert_predicate bin/"jump-high", :exist?
  end
end
