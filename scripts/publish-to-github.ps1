param(
	[string]$RepoName = "mhaid",
	[string]$GhExe = "C:\Users\user\.gemini\antigravity\brain\7e2fff51-f1df-47e0-84d7-a6443bb6fd07\scratch\gh\bin\gh.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $GhExe)) {
	$found = Get-Command gh -ErrorAction SilentlyContinue
	if ($found) {
		$GhExe = $found.Source
	} else {
		throw "GitHub CLI (gh.exe) not found at $GhExe or on PATH."
	}
}

Write-Host "Checking GitHub authentication..."
& $GhExe auth status
if ($LASTEXITCODE -ne 0) {
	Write-Host "Please sign in to GitHub in your browser:"
	& $GhExe auth login --hostname github.com --git-protocol https --web
	if ($LASTEXITCODE -ne 0) { throw "GitHub authentication failed." }
}

$owner = (& $GhExe api user --jq ".login").Trim()
if (-not $owner) { throw "Failed to resolve authenticated GitHub username." }
Write-Host "Authenticated as: $owner"

$repoFull = "$owner/$RepoName"
$exists = $true
try {
	& $GhExe repo view $repoFull 2>$null | Out-Null
	if ($LASTEXITCODE -ne 0) { $exists = $false }
} catch {
	$exists = $false
}

if (-not $exists) {
	Write-Host "Creating public repository $repoFull..."
	& $GhExe repo create $repoFull --public --description "Aidoku extensions ported from Mihon (WeebCentral, Comix, MangaDex, MangaFire)"
	if ($LASTEXITCODE -ne 0) { throw "Failed to create repository $repoFull." }
}

$remoteUrl = "https://github.com/$repoFull.git"
$currentRemote = git remote get-url origin 2>$null
if ($LASTEXITCODE -eq 0 -and $currentRemote) {
	git remote set-url origin $remoteUrl
} else {
	git remote add origin $remoteUrl
}

& $GhExe auth setup-git

Write-Host "Pushing main and gh-pages branches to $remoteUrl..."
git push -u origin main
if ($LASTEXITCODE -ne 0) { throw "Failed to push main branch." }
git push -u origin gh-pages
if ($LASTEXITCODE -ne 0) { throw "Failed to push gh-pages branch." }

Write-Host "Enabling GitHub Pages on gh-pages branch..."
try {
	& $GhExe api --method POST "repos/$repoFull/pages" -f "source[branch]=gh-pages" -f "source[path]=/" 2>$null | Out-Null
} catch {
	# May already exist (409 Conflict); ensure it points to gh-pages
	& $GhExe api --method PUT "repos/$repoFull/pages" -f "source[branch]=gh-pages" -f "source[path]=/" 2>$null | Out-Null
}

Write-Host ""
Write-Host "Done! Add one of these URLs in Aidoku (Settings -> Source Lists):"
Write-Host "  GitHub Pages: https://$owner.github.io/$RepoName/index.min.json"
Write-Host "  Raw GitHub:   https://raw.githubusercontent.com/$repoFull/gh-pages/index.min.json"
