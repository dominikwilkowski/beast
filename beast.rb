class Beast < Formula
  desc "An ASCII game built in rust in loving memory of the 1984 hit BEAST by Dan Baker, Alan Brown, Mark Hamilton and Derrick Shadel"
  homepage "https://github.com/dominikwilkowski/beast"
  # when updating version run `brew install --build-from-source --formula --HEAD ./beast.rb` to test locally first
  # make sure you remove rust from brew after
  url "https://github.com/dominikwilkowski/beast/archive/refs/tags/v1.0.1.tar.gz"
  sha256 "0748569a7ecc2fcb01805656b6e48cc0625c12e0fe2f832f9eb372216b097547"
  license "GPL-3.0-or-later"
  head "https://github.com/dominikwilkowski/beast.git", branch: "main"

  depends_on "rust" => :build

  def install
    cd "beast" do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    system bin/"beast", "--version"
  end
end