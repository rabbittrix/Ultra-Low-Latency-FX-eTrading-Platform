# PowerShell script to publish all crates to crates.io automatically
# Handles dependency management and publishes in correct order
#
# Bumps must be done first (package versions in Cargo.toml). This script converts
# path deps to version deps for publish, then restores path deps for local work.
#
# Usage: .\scripts\publish-all.ps1

# Get the script directory and project root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Change to project root
Push-Location $ProjectRoot

# Must match [package] version / [workspace.package] version for crates being published.
$VERSION = "0.1.3"

Write-Host ("=== Publishing crates to crates.io ({0}) ===" -f $VERSION) -ForegroundColor Cyan
Write-Host "Project root: $ProjectRoot" -ForegroundColor Gray
Write-Host ""
Write-Host "Publishes: fx-utils, fx-md, fx-risk, fx-core, fx-liquidity-graph," -ForegroundColor Yellow
Write-Host "           fx-pricing, fx-router, fx-gateway, fx-proto" -ForegroundColor Yellow
Write-Host ""

# Check if user is logged in
Write-Host "Checking cargo login status..." -ForegroundColor Cyan
# Check for credentials file (most reliable method)
$credentialPaths = @(
    "$env:USERPROFILE\.cargo\credentials.toml",
    "$env:USERPROFILE\.cargo\credentials"
)
$hasCredentials = $false
foreach ($path in $credentialPaths) {
    if (Test-Path $path) {
        $content = Get-Content $path -Raw -ErrorAction SilentlyContinue
        if ($content -and ($content -match "token" -or $content -match "crates-io")) {
            $hasCredentials = $true
            break
        }
    }
}

# If no credentials file, try a quick dry-run test
if (-not $hasCredentials) {
    Write-Host "  Verifying login with test publish..." -ForegroundColor Gray
    Push-Location "crates/fx-utils" -ErrorAction SilentlyContinue
    if ($?) {
        $testPublish = cargo publish --dry-run 2>&1 | Out-String
        Pop-Location
        if ($LASTEXITCODE -eq 0 -or $testPublish -notmatch "authentication|unauthorized|not logged|login") {
            $hasCredentials = $true
        }
    }
}

if (-not $hasCredentials) {
    Write-Host "Error: Not logged in to crates.io" -ForegroundColor Red
    Write-Host "Please run: cargo login <your-api-token>" -ForegroundColor Yellow
    Pop-Location
    exit 1
}
Write-Host "Logged in successfully, proceeding..." -ForegroundColor Green
Write-Host ""

# Function to revert all dependencies to path
function Set-DependenciesToPath {
    Write-Host "Reverting all dependencies to path dependencies..." -ForegroundColor Yellow
    
    # Function to revert dependencies in a Cargo.toml file
    function Update-CargoToml {
        param([string]$FilePath, [string]$CrateName, [string]$Version)
        
        if (-not (Test-Path $FilePath)) {
            return
        }
        
        $content = Get-Content $FilePath -Raw
        $original = $content
        
        # Determine path based on file location
        $relativePath = if ($FilePath -match "\\services\\") {
            "../../crates/$CrateName"
        }
        else {
            "../$CrateName"
        }
        
        # Revert version to path (literal Replace — avoids backtick-quote lexer issues).
        $content = $content.Replace(
            ('{0} = {{ version = "{1}" }}' -f $CrateName, $Version),
            ('{0} = {{ path = "{1}" }}' -f $CrateName, $relativePath)
        )
        $content = $content.Replace(
            ('{0} = {{ version = "{1}",' -f $CrateName, $Version),
            ('{0} = {{ path = "{1}",' -f $CrateName, $relativePath)
        )
        $cn = [regex]::Escape($CrateName)
        $ver = [regex]::Escape($Version)
        $content = [regex]::Replace(
            $content,
            ($cn + '\s*=\s*\{\s*version\s*=\s*"' + $ver + '"\s*\}'),
            ('{0} = {{ path = "{1}" }}' -f $CrateName, $relativePath)
        )
        
        if ($content -ne $original) {
            Set-Content -Path $FilePath -Value $content -NoNewline
        }
    }
    
    $VERSION = $script:VERSION
    
    # Revert in crates
    $cratesToCheck = @("fx-core", "fx-md", "fx-risk", "fx-pricing", "fx-router", "fx-gateway", "fx-proto", "fx-liquidity-graph", "fx-exchange", "fx-oms", "fx-ems", "fx-lp")
    foreach ($crate in $cratesToCheck) {
        $cratePath = "crates/$crate/Cargo.toml"
        if (Test-Path $cratePath) {
            Update-CargoToml -FilePath $cratePath -CrateName "fx-utils" -Version $VERSION
            Update-CargoToml -FilePath $cratePath -CrateName "fx-md" -Version $VERSION
            Update-CargoToml -FilePath $cratePath -CrateName "fx-risk" -Version $VERSION
            Update-CargoToml -FilePath $cratePath -CrateName "fx-core" -Version $VERSION
            Update-CargoToml -FilePath $cratePath -CrateName "fx-liquidity-graph" -Version $VERSION
            Update-CargoToml -FilePath $cratePath -CrateName "fx-oms" -Version $VERSION
        }
    }
    
    # Ensure services always use path dependencies (they're not published)
    # Services must use path deps because they're validated during crate publishing
    $services = Get-ChildItem -Path "services" -Directory -ErrorAction SilentlyContinue
    foreach ($service in $services) {
        $serviceToml = Join-Path $service.FullName "Cargo.toml"
        if (Test-Path $serviceToml) {
            $content = Get-Content $serviceToml -Raw
            $original = $content
            
            # Force all fx-* dependencies to use path (revert any version deps)
            $verEsc = [regex]::Escape($VERSION)
            foreach ($dep in @(
                'fx-utils', 'fx-md', 'fx-risk', 'fx-core', 'fx-pricing',
                'fx-router', 'fx-gateway', 'fx-proto', 'fx-liquidity-graph'
            )) {
                $pattern = [regex]::Escape($dep) + ' = \{ version = "' + $verEsc + '" \}'
                $replacement = $dep + ' = { path = "../../crates/' + $dep + '" }'
                $content = [regex]::Replace($content, $pattern, $replacement)
            }
            
            if ($content -ne $original) {
                Set-Content -Path $serviceToml -Value $content -NoNewline
            }
        }
    }
   
    Write-Host "Dependencies reverted." -ForegroundColor Green
}

# Function to update dependencies to versions for specific published crates
# Only updates crates that are about to be published (not all crates)
function Set-DependenciesToVersion {
    param([string]$Version, [string[]]$PublishedCrates, [string[]]$CratesToUpdate = @())
    
    Write-Host "Updating dependencies to version $Version for published crates: $($PublishedCrates -join ', ')" -ForegroundColor Yellow
    if ($CratesToUpdate.Count -gt 0) {
        Write-Host "  Updating dependencies in: $($CratesToUpdate -join ', ')" -ForegroundColor Gray
    }
    
    # Function to update dependencies in a Cargo.toml file
    function Update-CargoToml {
        param([string]$FilePath, [string]$CrateName, [string]$Version)
        
        if (-not (Test-Path $FilePath)) {
            return
        }
        
        $content = Get-Content $FilePath -Raw
        $original = $content
        
        # Update path to version (literal Replace — no backtick-quotes).
        $content = $content.Replace(
            ('{0} = {{ path = "../{0}" }}' -f $CrateName),
            ('{0} = {{ version = "{1}" }}' -f $CrateName, $Version)
        )
        $content = $content.Replace(
            ('{0} = {{ path = "../../crates/{0}" }}' -f $CrateName),
            ('{0} = {{ version = "{1}" }}' -f $CrateName, $Version)
        )
        $content = $content.Replace(
            ('{0} = {{ path = "../{0}",' -f $CrateName),
            ('{0} = {{ version = "{1}",' -f $CrateName, $Version)
        )
        $content = $content.Replace(
            ('{0} = {{ path = "../../crates/{0}",' -f $CrateName),
            ('{0} = {{ version = "{1}",' -f $CrateName, $Version)
        )
        $cn = [regex]::Escape($CrateName)
        $content = [regex]::Replace(
            $content,
            ($cn + '\s*=\s*\{\s*path\s*=\s*"\.\./' + $cn + '"\s*\}'),
            ('{0} = {{ version = "{1}" }}' -f $CrateName, $Version)
        )
        $content = [regex]::Replace(
            $content,
            ($cn + '\s*=\s*\{\s*path\s*=\s*"\.\./\.\./crates/' + $cn + '"\s*\}'),
            ('{0} = {{ version = "{1}" }}' -f $CrateName, $Version)
        )
        
        if ($content -ne $original) {
            Set-Content -Path $FilePath -Value $content -NoNewline
        }
    }
    
    # Only update dependencies in crates that are about to be published
    $cratesToCheck = if ($CratesToUpdate.Count -gt 0) { $CratesToUpdate } else { @("fx-core", "fx-md", "fx-risk", "fx-pricing", "fx-router", "fx-gateway", "fx-proto") }
    
    foreach ($crate in $cratesToCheck) {
        $cratePath = "crates/$crate/Cargo.toml"
        if (Test-Path $cratePath) {
            # Only update dependencies for crates that have been published
            foreach ($published in $PublishedCrates) {
                Update-CargoToml -FilePath $cratePath -CrateName $published -Version $Version
            }
        }
    }
    
    # NOTE: Services are NOT updated - they always use path dependencies
    # Services are part of the workspace but not published, so they must use path deps
   
    Write-Host "Dependencies updated." -ForegroundColor Green
}

# Function to wait for a crate to be available on crates.io
function Wait-ForCrateAvailable {
    param(
        [string]$CrateName,
        [string]$Version,
        [int]$MaxWaitSeconds = 60
    )
    
    Write-Host ("  Verifying {0}@{1} is available on crates.io..." -f $CrateName, $Version) -ForegroundColor Gray
    $checkInterval = 3
    $quickVerifySeconds = 15  # Quick verification window
    $maxAttempts = [math]::Floor($MaxWaitSeconds / $checkInterval)
    $quickAttempts = [math]::Floor($quickVerifySeconds / $checkInterval)
    
    # First, check if it's in search (use exact match to avoid false positives like "cfx-core" matching "fx-core")
    $quickCheck = cargo search --limit 10 $CrateName 2>&1 | Out-String
    # Match exact crate name at start of line (not a substring)
    $searchPattern = '(?m)^\s*{0} = "{1}"' -f [regex]::Escape($CrateName), [regex]::Escape($Version)
    $inSearch = $quickCheck -match $searchPattern    
    if ($inSearch) {
        Write-Host ("  {0}@{1} found in search, verifying resolution..." -f $CrateName, $Version) -ForegroundColor Gray
    }
    else {
        Write-Host ("  {0}@{1} not yet in search, waiting..." -f $CrateName, $Version) -ForegroundColor Gray
    }
    
    # Try quick resolution check first (faster path)
    for ($attempt = 1; $attempt -le $quickAttempts; $attempt++) {
        # Force cargo to update the index more aggressively
        Push-Location $ProjectRoot
        $null = cargo update 2>&1 | Out-Null
        Pop-Location
        
        # Quick resolution test using a temp Cargo.toml (array join, not a here-string).
        # Editor language services often mis-parse here-string markers in comments.
        $testManifest = @(
            '[package]'
            ('name = "test-resolve-{0}"' -f $CrateName)
            'version = "0.1.0"'
            'edition = "2021"'
            ''
            '[dependencies]'
            ('{0} = "{1}"' -f $CrateName, $Version)
        ) -join [Environment]::NewLine
        
        $testDir = Join-Path $env:TEMP ("cargo-test-{0}" -f (Get-Random))
        try {
            New-Item -ItemType Directory -Path $testDir -Force | Out-Null
            Set-Content -Path (Join-Path $testDir "Cargo.toml") -Value $testManifest -NoNewline
            New-Item -ItemType Directory -Path (Join-Path $testDir "src") -Force | Out-Null
            Set-Content -Path (Join-Path $testDir "src\main.rs") -Value "fn main() {}" -NoNewline
            
            Push-Location $testDir
            # Force cargo to update index before checking
            $null = cargo update 2>&1 | Out-Null
            # Use cargo check instead of cargo tree - it's more reliable for resolution
            $null = cargo check 2>&1 | Out-Null
            $exitCode = $LASTEXITCODE
            Pop-Location
            
            if ($exitCode -eq 0) {
                Write-Host ("  {0}@{1} is available and can be resolved!" -f $CrateName, $Version) -ForegroundColor Green
                Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue
                return $true
            }
        }
        catch {
            # Ignore errors
        }
        finally {
            Pop-Location -ErrorAction SilentlyContinue
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        if ($attempt -lt $quickAttempts) {
            Start-Sleep -Seconds $checkInterval
        }
    }
    
    # If found in search but can't resolve quickly, wait longer for indexing
    # Crates.io indexing can take time, so we need to be patient
    if ($inSearch) {
        Write-Host ("  {0}@{1} found in search but not yet resolvable, waiting for indexing..." -f $CrateName, $Version) -ForegroundColor Yellow
        $waited = $quickVerifySeconds
        
        # Continue waiting for resolution (up to remaining time)
        for ($attempt = $quickAttempts + 1; $attempt -le $maxAttempts; $attempt++) {
            Start-Sleep -Seconds $checkInterval
            $waited += $checkInterval
            
            # Try resolution again
            $testManifest = @(
                '[package]'
                ('name = "test-resolve-{0}"' -f $CrateName)
                'version = "0.1.0"'
                'edition = "2021"'
                ''
                '[dependencies]'
                ('{0} = "{1}"' -f $CrateName, $Version)
            ) -join [Environment]::NewLine
            
            $testDir = Join-Path $env:TEMP ("cargo-test-{0}" -f (Get-Random))
            try {
                New-Item -ItemType Directory -Path $testDir -Force | Out-Null
                Set-Content -Path (Join-Path $testDir "Cargo.toml") -Value $testManifest -NoNewline
                New-Item -ItemType Directory -Path (Join-Path $testDir "src") -Force | Out-Null
                Set-Content -Path (Join-Path $testDir "src\main.rs") -Value "fn main() {}" -NoNewline
                
                Push-Location $testDir
                # Force cargo to update index before checking
                $null = cargo update 2>&1 | Out-Null
                # Use cargo check instead of cargo tree - it's more reliable for resolution
                $null = cargo check 2>&1 | Out-Null
                $exitCode = $LASTEXITCODE
                Pop-Location
                
                if ($exitCode -eq 0) {
                    Write-Host ("  {0}@{1} is now available and can be resolved!" -f $CrateName, $Version) -ForegroundColor Green
                    Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue
                    return $true
                }
            }
            catch {
                # Ignore errors
            }
            finally {
                Pop-Location -ErrorAction SilentlyContinue
                Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue
            }
            
            if ($attempt % 3 -eq 0) {
                Write-Host ("    Still waiting for indexing... ({0}/{1} seconds)" -f $waited, $MaxWaitSeconds) -ForegroundColor DarkGray
            }
        }
        
        # If we've waited the full time and still can't resolve, this is a problem
        Write-Host ("  Error: {0}@{1} found in search but not resolvable after {2} seconds" -f $CrateName, $Version, $MaxWaitSeconds) -ForegroundColor Red
        Write-Host "  This may indicate an indexing delay or the crate may not be fully published" -ForegroundColor Red
        Write-Host "  Cannot proceed - dependency must be resolvable before publishing dependent crates" -ForegroundColor Red
        return $false
    }
    
    # If not in search, continue waiting for the full duration
    $waited = $quickVerifySeconds
    for ($attempt = $quickAttempts + 1; $attempt -le $maxAttempts; $attempt++) {
        Start-Sleep -Seconds $checkInterval
        $waited += $checkInterval
        
        # Check search again (use exact match)
        $check = cargo search --limit 10 $CrateName 2>&1 | Out-String
        # Match exact crate name at start of line (not a substring)
        $searchPattern = '(?m)^\s*{0} = "{1}"' -f [regex]::Escape($CrateName), [regex]::Escape($Version)
        if ($check -match $searchPattern) {
            Write-Host ("  {0}@{1} now found in search (may still be indexing)" -f $CrateName, $Version) -ForegroundColor Yellow
            return $true
        }
        
        if ($attempt % 3 -eq 0) {
            Write-Host ("    Still waiting... ({0}/{1} seconds)" -f $waited, $MaxWaitSeconds) -ForegroundColor DarkGray
        }
    }
    
    Write-Host ("  Warning: {0}@{1} not found after {2} seconds" -f $CrateName, $Version, $MaxWaitSeconds) -ForegroundColor Yellow
    return $false
}

# Function to publish a crate
function Publish-Crate {
    param(
        [string]$CrateName,
        [string]$CratePath
    )
    
    Write-Host "Publishing $CrateName..." -ForegroundColor Yellow
    
    if (-not (Test-Path $CratePath)) {
        Write-Host "Error: Crate path not found: $CratePath" -ForegroundColor Red
        return $false
    }
    
    Push-Location $CratePath
    
    try {
        # Check if crate already exists on crates.io (use exact match)
        Write-Host "  Checking if crate already exists..." -ForegroundColor Gray
        $checkExists = cargo search --limit 10 $CrateName 2>&1 | Out-String
        # Match exact crate name (not a substring like "cfx-core" matching "fx-core")
        if ($checkExists -match ('^\s*{0} = "{1}"' -f [regex]::Escape($CrateName), [regex]::Escape($VERSION))) {
            Write-Host ("  Crate {0}@{1} already exists on crates.io, skipping..." -f $CrateName, $VERSION) -ForegroundColor Yellow
            # Even if it exists, we MUST wait for it to be resolvable before dependent crates can use it
            # This is critical - don't skip the wait!
            Write-Host ("  Waiting for {0}@{1} to be resolvable (required for dependent crates)..." -f $CrateName, $VERSION) -ForegroundColor Gray
            $isAvailable = Wait-ForCrateAvailable -CrateName $CrateName -Version $VERSION -MaxWaitSeconds 90
            if (-not $isAvailable) {
                Write-Host ("  Error: {0}@{1} exists but is not resolvable after 90 seconds" -f $CrateName, $VERSION) -ForegroundColor Red
                Write-Host "  Cannot proceed - dependent crates need this to be resolvable" -ForegroundColor Red
                return $false
            }
            Write-Host ("  {0}@{1} is now resolvable" -f $CrateName, $VERSION) -ForegroundColor Green
            return $true
        }
        
        # Dry run first (allow dirty files since we're modifying Cargo.toml)
        Write-Host "  Running dry-run..." -ForegroundColor Gray
        $dryRun = cargo publish --dry-run --allow-dirty 2>&1
        if ($LASTEXITCODE -ne 0) {
            # Check if it's because crate already exists
            if ($dryRun -match "already exists") {
                Write-Host ("  Crate {0}@{1} already exists on crates.io, skipping..." -f $CrateName, $VERSION) -ForegroundColor Yellow
                # Even if it exists, we MUST wait for it to be resolvable
                Write-Host ("  Waiting for {0}@{1} to be resolvable (required for dependent crates)..." -f $CrateName, $VERSION) -ForegroundColor Gray
                $isAvailable = Wait-ForCrateAvailable -CrateName $CrateName -Version $VERSION -MaxWaitSeconds 90
                if (-not $isAvailable) {
                    Write-Host ("  Error: {0}@{1} exists but is not resolvable after 90 seconds" -f $CrateName, $VERSION) -ForegroundColor Red
                    return $false
                }
                Write-Host ("  {0}@{1} is now resolvable" -f $CrateName, $VERSION) -ForegroundColor Green
                return $true
            }
            Write-Host "  Dry-run failed!" -ForegroundColor Red
            Write-Host $dryRun -ForegroundColor Red
            return $false
        }
        
        Write-Host "  Dry-run successful. Publishing..." -ForegroundColor Gray
        
        # Publish (automatic, no confirmation needed, allow dirty files)
        $publish = cargo publish --allow-dirty 2>&1
        if ($LASTEXITCODE -ne 0) {
            # Check if it's because crate already exists
            if ($publish -match "already exists") {
                Write-Host ("  Crate {0}@{1} already exists on crates.io, skipping..." -f $CrateName, $VERSION) -ForegroundColor Yellow
                # Even if it exists, we MUST wait for it to be resolvable
                Write-Host ("  Waiting for {0}@{1} to be resolvable (required for dependent crates)..." -f $CrateName, $VERSION) -ForegroundColor Gray
                $isAvailable = Wait-ForCrateAvailable -CrateName $CrateName -Version $VERSION -MaxWaitSeconds 90
                if (-not $isAvailable) {
                    Write-Host ("  Error: {0}@{1} exists but is not resolvable after 90 seconds" -f $CrateName, $VERSION) -ForegroundColor Red
                    return $false
                }
                Write-Host ("  {0}@{1} is now resolvable" -f $CrateName, $VERSION) -ForegroundColor Green
                return $true
            }
            Write-Host "  Publishing failed!" -ForegroundColor Red
            Write-Host $publish -ForegroundColor Red
            return $false
        }
        
        Write-Host "  Successfully published $CrateName!" -ForegroundColor Green
        
        # Wait for the crate to be indexed before continuing
        Wait-ForCrateAvailable -CrateName $CrateName -Version $VERSION -MaxWaitSeconds 60
        
        return $true
    }
    finally {
        Pop-Location
    }
}

# Step 1: Revert all dependencies to path
Set-DependenciesToPath
Write-Host ""

# Track published crates
$publishedCrates = @()

# Step 2: Publish fx-utils (no dependencies)
Write-Host "=== Step 1: Publishing fx-utils ===" -ForegroundColor Cyan
$result = Publish-Crate -CrateName "fx-utils" -CratePath "crates/fx-utils"
if (-not $result) {
    Write-Host "Failed to publish fx-utils. Stopping." -ForegroundColor Red
    Pop-Location
    exit 1
}
$publishedCrates += "fx-utils"
# Wait for fx-utils to be indexed (if it was just published)
Wait-ForCrateAvailable -CrateName "fx-utils" -Version $VERSION -MaxWaitSeconds 30
Write-Host ""

# Step 3: Publish Group 1 (fx-md, fx-risk, fx-core, fx-liquidity-graph)
# Update dependencies one crate at a time, just before publishing
Write-Host "=== Step 2: Publishing Group 1 (fx-md, fx-risk, fx-core, fx-liquidity-graph) ===" -ForegroundColor Cyan
$group1 = @(
    @{Name = "fx-md"; Path = "crates/fx-md" },
    @{Name = "fx-risk"; Path = "crates/fx-risk" },
    @{Name = "fx-core"; Path = "crates/fx-core" },
    @{Name = "fx-liquidity-graph"; Path = "crates/fx-liquidity-graph" }
)

foreach ($crate in $group1) {
    # Update dependencies for this crate only, just before publishing
    Set-DependenciesToVersion -Version $VERSION -PublishedCrates $publishedCrates -CratesToUpdate @($crate.Name)
    
    # Wait for all dependencies to be available before publishing
    if ($publishedCrates.Count -gt 0) {
        Write-Host "  Verifying dependencies are available..." -ForegroundColor Gray
        # Force cargo to update its index first
        Write-Host "  Updating cargo index..." -ForegroundColor DarkGray
        Push-Location $ProjectRoot
        $null = cargo update 2>&1 | Out-Null
        Pop-Location
        
        foreach ($dep in $publishedCrates) {
            # Wait longer for dependencies that were just published (they need indexing time)
            $wasJustPublished = $dep -eq $publishedCrates[-1]
            $waitTime = if ($wasJustPublished) { 90 } else { 60 }
            $isAvailable = Wait-ForCrateAvailable -CrateName $dep -Version $VERSION -MaxWaitSeconds $waitTime
            if (-not $isAvailable) {
                Write-Host ("  Error: {0}@{1} is not available after waiting. Cannot proceed." -f $dep, $VERSION) -ForegroundColor Red
                Write-Host ("Failed to verify dependencies for {0}. Stopping." -f $crate.Name) -ForegroundColor Red
                Pop-Location
                exit 1
            }
        }
    }
    
    $result = Publish-Crate -CrateName $crate.Name -CratePath $crate.Path
    if (-not $result) {
        Write-Host ("Failed to publish {0}. Stopping." -f $crate.Name) -ForegroundColor Red
        Pop-Location
        exit 1
    }
    $publishedCrates += $crate.Name
    
    # After publishing (or skipping), wait for the crate to be resolvable
    # This ensures dependent crates can find it (especially important for fx-core -> fx-router)
    Write-Host ("  Verifying {0}@{1} is resolvable after publish/skip..." -f $crate.Name, $VERSION) -ForegroundColor Gray
    $isResolvable = Wait-ForCrateAvailable -CrateName $crate.Name -Version $VERSION -MaxWaitSeconds 90
    if (-not $isResolvable) {
        Write-Host ("  Error: {0}@{1} is not resolvable after waiting. Cannot proceed." -f $crate.Name, $VERSION) -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Write-Host ""
}
Write-Host ""

# Step 4: Publish Group 2 (fx-pricing, fx-router, fx-gateway)
# Update dependencies one crate at a time, just before publishing
Write-Host "=== Step 3: Publishing Group 2 (fx-pricing, fx-router, fx-gateway) ===" -ForegroundColor Cyan
$group2 = @(
    @{Name = "fx-pricing"; Path = "crates/fx-pricing" },
    @{Name = "fx-router"; Path = "crates/fx-router" },
    @{Name = "fx-gateway"; Path = "crates/fx-gateway" }
)

foreach ($crate in $group2) {
    # Update dependencies for this crate only, just before publishing
    Set-DependenciesToVersion -Version $VERSION -PublishedCrates $publishedCrates -CratesToUpdate @($crate.Name)
    
    # Wait for all dependencies to be available before publishing
    if ($publishedCrates.Count -gt 0) {
        Write-Host "  Verifying dependencies are available..." -ForegroundColor Gray
        # Force cargo to update its index first
        Write-Host "  Updating cargo index..." -ForegroundColor DarkGray
        Push-Location $ProjectRoot
        $null = cargo update 2>&1 | Out-Null
        Pop-Location
        
        foreach ($dep in $publishedCrates) {
            # Wait longer for dependencies that were just published (they need indexing time)
            $wasJustPublished = $dep -eq $publishedCrates[-1]
            $waitTime = if ($wasJustPublished) { 90 } else { 60 }
            $isAvailable = Wait-ForCrateAvailable -CrateName $dep -Version $VERSION -MaxWaitSeconds $waitTime
            if (-not $isAvailable) {
                Write-Host ("  Error: {0}@{1} is not available after waiting. Cannot proceed." -f $dep, $VERSION) -ForegroundColor Red
                Write-Host ("Failed to verify dependencies for {0}. Stopping." -f $crate.Name) -ForegroundColor Red
                Pop-Location
                exit 1
            }
        }
    }
    
    $result = Publish-Crate -CrateName $crate.Name -CratePath $crate.Path
    if (-not $result) {
        Write-Host ("Failed to publish {0}. Stopping." -f $crate.Name) -ForegroundColor Red
        Pop-Location
        exit 1
    }
    $publishedCrates += $crate.Name
    
    # After publishing (or skipping), wait for the crate to be resolvable
    # This ensures dependent crates can find it (especially important for fx-core -> fx-router)
    Write-Host ("  Verifying {0}@{1} is resolvable after publish/skip..." -f $crate.Name, $VERSION) -ForegroundColor Gray
    $isResolvable = Wait-ForCrateAvailable -CrateName $crate.Name -Version $VERSION -MaxWaitSeconds 90
    if (-not $isResolvable) {
        Write-Host ("  Error: {0}@{1} is not resolvable after waiting. Cannot proceed." -f $crate.Name, $VERSION) -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Write-Host ""
}
Write-Host ""

# Update dependencies for fx-proto before publishing
Set-DependenciesToVersion -Version $VERSION -PublishedCrates $publishedCrates -CratesToUpdate @("fx-proto")

# Wait for all dependencies to be available before publishing fx-proto
if ($publishedCrates.Count -gt 0) {
    Write-Host "Verifying dependencies are available..." -ForegroundColor Gray
    # Force cargo to update its index first
    Write-Host "Updating cargo index..." -ForegroundColor DarkGray
    $null = cargo update 2>&1 | Out-Null
    
    foreach ($dep in $publishedCrates) {
        # Wait longer for dependencies that were just published
        $wasJustPublished = $dep -eq $publishedCrates[-1]
        $waitTime = if ($wasJustPublished) { 90 } else { 60 }
        $isAvailable = Wait-ForCrateAvailable -CrateName $dep -Version $VERSION -MaxWaitSeconds $waitTime
        if (-not $isAvailable) {
            Write-Host ("Error: {0}@{1} is not available after waiting. Cannot proceed." -f $dep, $VERSION) -ForegroundColor Red
            Write-Host "Failed to verify dependencies for fx-proto. Stopping." -ForegroundColor Red
            Pop-Location
            exit 1
        }
    }
}
Write-Host ""

# Step 5: Publish fx-proto
Write-Host "=== Step 4: Publishing fx-proto ===" -ForegroundColor Cyan
$result = Publish-Crate -CrateName "fx-proto" -CratePath "crates/fx-proto"
if (-not $result) {
    Write-Host "Failed to publish fx-proto." -ForegroundColor Red
    Pop-Location
    exit 1
}
$publishedCrates += "fx-proto"

Write-Host ""
Write-Host "Restoring path dependencies for local development..." -ForegroundColor Cyan
Set-DependenciesToPath

Write-Host ""
Write-Host ("=== All crates published successfully! ({0}) ===" -f $VERSION) -ForegroundColor Green
Write-Host ""
Write-Host "Verify on crates.io:" -ForegroundColor Cyan
Write-Host "  https://crates.io/crates/fx-utils" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-md" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-risk" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-core" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-liquidity-graph" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-pricing" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-router" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-gateway" -ForegroundColor White
Write-Host "  https://crates.io/crates/fx-proto" -ForegroundColor White

Pop-Location
