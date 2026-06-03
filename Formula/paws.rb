class Paws < Formula
  desc "Play games in your terminal while your AI coding agent works"
  homepage "https://github.com/interesting-vibe-coding/paws"
  license "MIT"

  stable do
    url "https://github.com/interesting-vibe-coding/paws/archive/refs/tags/v0.3.1.tar.gz"
    sha256 "238b6b02734aea3d1b3aa316db23b52f14437e3cd92302e8a9b0f2bef8910f58"
    version "0.3.1"
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
