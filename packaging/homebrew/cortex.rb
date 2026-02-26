class Cortex < Formula
  desc "Portable Brain + OpenAI-compatible Cortex proxy"
  homepage "https://github.com/vinzify/Cortex-portable-brain"
  version "0.1.0-alpha.1"
  url "https://github.com/vinzify/Cortex-portable-brain/releases/download/v0.1.0-alpha.1/cortex-app-macos-arm64"
  sha256 "REPLACE_WITH_RELEASE_SHA256"
  license "MIT"

  def install
    bin.install "cortex-app-macos-arm64" => "cortex"
  end

  test do
    assert_match "Portable Brain", shell_output("#{bin}/cortex --help")
  end
end
