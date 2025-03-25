use anyhow::Result;
use git2::{DiffOptions, Repository as Git2Repo, Status, StatusOptions};
use std::path::Path;

pub struct Repository {
    repo: Git2Repo,
    verbose: bool,
}

impl Repository {
    pub fn open_current_dir(verbose: bool) -> Result<Self> {
        let path = std::env::current_dir()?;
        Self::open(&path, verbose)
    }

    pub fn open(path: &Path, verbose: bool) -> Result<Self> {
        let repo = Git2Repo::open(path)?;
        Ok(Self { repo, verbose })
    }

    pub fn get_branch_name(&self) -> Result<String> {
        let head = self.repo.head()?;
        if head.is_branch() {
            head.shorthand()
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("Failed to get branch name"))
        } else {
            Ok("detached-head".to_string())
        }
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let head = self.repo.head().ok();
        // Use as_ref() to borrow the Option's contents rather than taking ownership
        let tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());
        
        if self.verbose && head.is_none() {
            println!("Debug: Repository has no HEAD commit yet");
        }
        
        let mut options = DiffOptions::new();
        let diff = self.repo.diff_tree_to_index(tree.as_ref(), None, Some(&mut options))?;
        
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
            if matches!(line.origin(), 'H' | '+' | '-') {
                if let Ok(content) = std::str::from_utf8(line.content()) {
                    diff_text.push_str(content);
                }
            }
            true
        })?;
        
        if diff_text.is_empty() && self.verbose {
            self.debug_staging_status()?;
        }
        
        Ok(diff_text)
    }
    
    fn debug_staging_status(&self) -> Result<()> {
        println!("Debug: No changes detected in staging area. Checking repository status:");
        
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        
        if statuses.is_empty() {
            println!("Debug: No changes in the repository");
            return Ok(());
        }
        
        println!("Debug: Found {} changed files:", statuses.len());
        for entry in statuses.iter() {
            let status = entry.status();
            let is_staged = status.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED);
            let path = String::from_utf8_lossy(entry.path_bytes());
            
            println!("Debug: {} - staged: {}, status: {:?}", path, is_staged, status);
        }
        
        Ok(())
    }
}
