class Paws < Formula
  desc "Play games in your terminal while your AI coding agent works"
  homepage "https://github.com/interesting-vibe-coding/paws"
  license "MIT"

  stable do
    url "https://github.com/interesting-vibe-coding/paws/archive/refs/tags/v0.3.0.tar.gz"
    sha256 "3f37c0276d062cd8799b1d71892ca1d6448d60be939c28885637859c0c3e181e"
    version "0.3.0"
  end

  head "https://github.com/interesting-vibe-coding/paws.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"paws", "--list"
  end
end
