class Paws < Formula
  desc "Play games in your terminal while your AI coding agent works"
  homepage "https://github.com/interesting-vibe-coding/paws"
  license "MIT"

  stable do
    url "https://github.com/interesting-vibe-coding/paws/archive/refs/tags/v0.3.0.tar.gz"
    sha256 ""  # TODO: fill after tagging v0.3.0
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
