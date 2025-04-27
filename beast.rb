class Beast < Formula
  desc "An ASCII game built in rust in loving memory of the 1984 hit BEAST by Dan Baker, Alan Brown, Mark Hamilton and Derrick Shadel"
  homepage "https://github.com/dominikwilkowski/beast"
  # when updating version run `brew install --build-from-source --formula --HEAD ./beast.rb` to test locally first
  url "https://github.com/dominikwilkowski/beast/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "40abb007b829dd2e0e2cc0598a00c512ad8dd820eb31b2f9b0a8f2b89fa08d79"
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