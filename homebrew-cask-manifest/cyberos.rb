# TASK-APP-006: Homebrew Cask manifest for CyberOS (draft - lives in-repo; the eventual
# listing is a Stephen-gated PR to the external homebrew-cask monorepo, never automated:
# spec 1 #5/#6, structurally enforced by release-pkgmgr-pr.yml's standing guard).
#
# sha256 is an UNAMBIGUOUS placeholder, never a plausible-looking fake - the prep job in
# release-pkgmgr-pr.yml re-derives version + sha256 from the actual GitHub Release artifact
# (AC #5), and brew audit itself rejects the placeholder if anyone tries to submit it as-is.
#
# zap trash: paths are PLAUSIBLE CANDIDATES derived from the confirmed Tauri identifier
# (os.cyberskill.world.desktop), NOT confirmed-observed paths - spec 1 #9 requires a real
# `brew uninstall --zap` test against an installed copy before this stanza is final
# (answer sheet records the result; brew audit/style cannot catch a wrong path here).
cask "cyberos" do
  version "1.0.0"
  sha256 "REPLACE_WITH_SHA256_OF_RELEASE_DMG" # computed per spec 6, not hand-typed

  url "https://github.com/cyberskill-official/cyberos/releases/download/v#{version}/CyberOS_#{version}_universal.dmg"
  name "CyberOS"
  desc "CyberSkill's desktop client — Turn Your Will Into Real"
  homepage "https://os.cyberskill.world/"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "CyberOS.app"

  zap trash: [
    "~/Library/Application Support/CyberOS",
    "~/Library/Preferences/os.cyberskill.world.desktop.plist",
  ]
end
