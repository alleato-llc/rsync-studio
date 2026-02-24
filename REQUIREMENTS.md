## A Mac OSX & Linux GUI for Rsync
1. Rust Backend + Typescript Front Backend
2. Should be able to manage Rsync Jobs Definitions (CRUD)
3. Should be able to schedule on Linux via cron and launchd on Mac OSX
4. Should be able to manually start rsync jobs
5. Should be able to see progress bar (for background jobs and manually started jobs)
6. Should be able paste in rsync commands and explain to user what each argument is
7. As user is customizing the job an rsync command is shown for the user to see
8. Should be able to stop manually started and scheduled job
9. Should be able to export jobs
10. Should support both local and remote storage
11. Should be able to support ssh and rsync protocols for tunneling
12. In addition to progress bar should be able to see logs
13. Should be able to export logs
14. Thinking we have a simple vs advanced mode or profiles (see 17 - maybe related or unrelated need to think through UX)
15. Preflight Validation - Before run:
- Check source exists
- Check destination writable
- Check disk space (optional but powerful)
- Check SSH connectivity
16. Snapshot-style Backups (rsync + hardlinks) via --link-dest. When using snapshot mode:
- Do NOT just pass --link-dest randomly.
- You must:
- Create new timestamp directory
- Pass --link-dest=previous_snapshot
- Update latest symlink
- That’s how proper incremental backups work.
16. Core Aspects of Job Definitions
- Source & Destination Types
- Archive Mode
- Delete Mode --delete (should require confirmation)
- Dry Run (should be dedicated UX - "Preview")
- Exclude / Include Rules
- SSH Configuration
- Compression
- Scheduling
- Versioned Backups
- Partial Resume
- "--bwlimit" (low priority)
- Progress: "--progress / --info=progress2"
- Snapshot-style Backups (rsync + hardlinks) via --link-dest
- Test ssh connection
17. UX Provides Backup Mode Templates that the user can override
- Backup Type 1 — Mirror (Simple): `-a --delete --partial`
- Backup Type 2 — Versioned (Safer): `-a --delete --backup --backup-dir=archive/`
- Backup Type 3 — Snapshot (Power Mode): `-a --delete --link-dest=previous_snapshot`
18. Automatic Management of:
- Snapshot folder naming
- latest symlink
- retention policy (Retention example: Keep last 7 daily, Keep 4 weekly, Keep 6 monthly) which can be configured
19. Future idea: Restore Mode
20. Future idea: Remote Management - Desktop Client Manges Configuration of Linux Server
21. Modern Object Oriented Design Principles
- All Rsync Commands are encpauslated to an `ProcessRsyncClient` which implements `RsyncClient`
- MUST HAVE: EXTENSIVE UNIT TESTS which use a `TestRsyncClient` (see point 22) which also implements `RsyncClientProtocol`. 
22. `TestRsyncClient` 
- Has a `TestFileSystem` abstraction internally which stores state of supported directories and valid files (which are configured on init pointing to real files and folderes) can be random temp files (with random bytes) and folders 
- records commands invoked to validate integration (stored internally as an array)
- updates `TestFileSystem` state based on invoked commands (e.g. if we back up to /tmp then `TestFileSystem` has that state). Internally `TestRsyncClient` replicates the behavior of `RsyncClient` 
- In test we track both `TestRsyncClient` and `TestFileSystem`. So to check a backup after running a command we query `TestFileSystem` to ensure that `TestRsyncClient` properly communicated with the file system 
- We should unit tests both of these test components
- But the expectation is that (a) ** we store no business logic in view layer** and (b) **store all business logic in service layer** and then write "integration" tests validating the service-layer abstractions and their integration with the `RsyncClientProtocol` implementation
23. App should store all backup invocations and all logs at configured location to be able to view historical backups
