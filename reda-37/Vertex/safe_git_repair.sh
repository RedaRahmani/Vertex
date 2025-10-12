#!/bin/bash
set -euo pipefail

trap 'echo >&2 "[FAIL] Line $LINENO exited with status $?"; exit 1' ERR

# Print diagnostics function
print_diagnostics() {
    echo "Current state:"
    pwd
    git --version
    echo "TOP: $(git rev-parse --show-toplevel 2>/dev/null || echo NO_REPO)"
    echo "GITDIR: $(git rev-parse --git-dir 2>/dev/null || echo NONE)"
    [[ -d /home/.git ]] && echo "OUTER_REPO=/home/.git" || echo "OUTER_REPO=NONE"
    [[ -d .git ]] && echo "INNER_REPO=$(pwd)/.git" || echo "INNER_REPO=NONE"
}

# Dry run mode
DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    echo "DRY RUN MODE - No changes will be made"
    DRY_RUN=true
fi

# Restore outer repo mode
if [[ "${1:-}" == "--restore-outer" ]]; then
    echo "Attempting to restore outer repository..."
    LATEST_BACKUP=$(ls -t /home/.git.backup_* 2>/dev/null | head -n1 || echo "NONE")
    if [[ "$LATEST_BACKUP" == "NONE" ]]; then
        echo "No backup found to restore"
        exit 1
    fi
    
    read -r -p "Restore $LATEST_BACKUP to /home/.git? [y/N] " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        if [ "$DRY_RUN" = false ]; then
            mv "$LATEST_BACKUP" /home/.git
            echo "Restored outer repository"
        else
            echo "[DRY RUN] Would restore $LATEST_BACKUP to /home/.git"
        fi
    fi
    exit 0
fi

# Change to Vertex directory
cd ~/Vertex
echo "Changed to $(pwd)"

# Print initial diagnostics
print_diagnostics

# Step A: Create backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="/home/$USER/Vertex_backup_${TIMESTAMP}.tar.gz"
echo "Creating backup at $BACKUP_FILE"
if [ "$DRY_RUN" = false ]; then
    tar --exclude='.vscode-server' --exclude='.vscode-remote' -czf "$BACKUP_FILE" .
    echo "Backup created"
else
    echo "[DRY RUN] Would create backup at $BACKUP_FILE"
fi

# Step B: Handle outer repo
if [[ -d /home/.git ]]; then
    echo "Found outer repository at /home/.git"
    du -sh /home/.git || true
    read -r -p "Move outer repository to /home/.git.backup_${TIMESTAMP}? [y/N] " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        if [ "$DRY_RUN" = false ]; then
            if mv /home/.git "/home/.git.backup_${TIMESTAMP}" 2>/dev/null; then
                echo "Moved outer repository to backup"
            else
                echo "Normal move failed; trying with sudo…"
                sudo mv /home/.git "/home/.git.backup_${TIMESTAMP}"
                echo "Moved outer repository with sudo"
            fi
        else
            echo "[DRY RUN] Would move /home/.git to /home/.git.backup_${TIMESTAMP}"
        fi
    fi
fi

# Step C: Ensure repo is properly rooted
if [[ ! -d .git ]]; then
    echo "No .git directory found in $(pwd)"
    read -r -p "Initialize new git repository here? [y/N] " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        if [ "$DRY_RUN" = false ]; then
            git init
            echo "Initialized new git repository"
        else
            echo "[DRY RUN] Would initialize new git repository"
        fi
    fi
fi

# Verify repository root
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo "NONE")
if [[ "$REPO_ROOT" != "$HOME/Vertex" ]]; then
    echo "ERROR: Repository root is not at ~/Vertex"
    echo "Current root: $REPO_ROOT"
    exit 1
fi

# Step D: Normalize index
echo "Current git status:"
git status

read -r -p "Proceed with index normalization? [y/N] " response
if [[ "$response" =~ ^[Yy]$ ]]; then
    if [ "$DRY_RUN" = false ]; then
        git rm -r --cached . || true
        git add -A
        echo "Staged adds:"
        git diff --cached --name-status | sed -n '1,200p'
        echo "Total staged:"
        git diff --cached --name-only | wc -l

        read -r -p "Commit these changes? [y/N] " commit_response
        if [[ "$commit_response" =~ ^[Yy]$ ]]; then
            git commit -m "Track full project from Vertex root (safe repair)"
        fi
    else
        echo "[DRY RUN] Would normalize index and commit changes"
    fi
fi

# Step E: Remote setup
if ! git remote get-url origin &>/dev/null; then
    read -r -p "Set remote origin? Enter repo URL or press enter to skip: " repo_url
    if [[ -n "$repo_url" ]]; then
        if [ "$DRY_RUN" = false ]; then
            git remote add origin "$repo_url"
            git branch -M main
            read -r -p "Push to remote? [y/N] " push_response
            if [[ "$push_response" =~ ^[Yy]$ ]]; then
                git push -u origin main
            fi
        else
            echo "[DRY RUN] Would set up remote and push"
        fi
    fi
fi

# Step F: Gitignore setup
if [[ ! -f .gitignore ]]; then
    echo "No .gitignore found"
    read -r -p "Create standard .gitignore? [y/N] " response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        if [ "$DRY_RUN" = false ]; then
            cat > .gitignore << 'EOL'
node_modules/
.next/
dist/
target/
*.log
.env*
.DS_Store
.vscode/
.idea/
EOL
            git add -A
            echo "Staged adds after .gitignore:"
            git diff --cached --name-status | sed -n '1,200p'
            echo "Total staged:"
            git diff --cached --name-only | wc -l

            read -r -p "Commit .gitignore? [y/N] " commit_response
            if [[ "$commit_response" =~ ^[Yy]$ ]]; then
                git commit -m "Add sane .gitignore"
            fi
        else
            echo "[DRY RUN] Would create .gitignore and commit"
        fi
    fi
fi

echo "Done. Repository state:"
print_diagnostics