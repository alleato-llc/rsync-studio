export const config = {
  // Site metadata
  site: {
    title: "Rsync Studio",
    description:
      "A cross-platform desktop application for managing rsync backup jobs. Desktop GUI and Terminal UI, powered by Rust.",
  },

  // Theme settings
  theme: {
    enabled: true,
    default: "system" as "light" | "dark" | "system",
  },

  // Hero section
  hero: {
    appName: "Rsync Studio",
    tagline: "Backups made ",
    accentWord: "visual.",
    subtitle:
      "A cross-platform rsync management tool with a desktop GUI and terminal UI. Built with Rust for reliability.",
  },

  // Feature pillars (3 large cards)
  pillars: [
    {
      icon: "monitor",
      title: "Visual Rsync Management",
      description:
        "Configure rsync jobs through an intuitive interface. See a live command preview as you adjust options — no more guessing what flags do.",
    },
    {
      icon: "shield",
      title: "Automated Backups",
      description:
        "Schedule jobs with cron expressions or intervals. Mirror, versioned, and snapshot modes with retention policies keep your data safe automatically.",
    },
    {
      icon: "layers",
      title: "Two Frontends, One Core",
      description:
        "Desktop GUI for daily use, Terminal UI for headless servers. Both share the same Rust core, database, and job executor.",
    },
  ],

  // Feature grid (12 cards with tags)
  features: [
    {
      icon: "terminal",
      title: "Live Command Preview",
      description:
        "See the exact rsync command as you configure options. Every flag change is reflected instantly.",
      tag: "GUI" as const,
    },
    {
      icon: "copy",
      title: "Backup Modes",
      description:
        "Mirror, Versioned, and Snapshot modes. Snapshots use --link-dest for space-efficient incremental backups.",
      tag: "Both" as const,
    },
    {
      icon: "lock",
      title: "SSH Configuration",
      description:
        "Full SSH support with custom ports, identity files, and host key checking options.",
      tag: "Both" as const,
    },
    {
      icon: "clock",
      title: "Job Scheduling",
      description:
        "Cron expressions and interval-based scheduling. Run jobs automatically on your schedule.",
      tag: "Both" as const,
    },
    {
      icon: "chart",
      title: "Run Statistics",
      description:
        "Track files transferred, data volume, duration, speedup ratio, and time saved across all runs.",
      tag: "Both" as const,
    },
    {
      icon: "wifi",
      title: "NAS Detection",
      description:
        "Auto-detects network filesystems (SMB, NFS, AFP) and enables --size-only to prevent re-transfers.",
      tag: "Both" as const,
    },
    {
      icon: "search",
      title: "Dry Run Analysis",
      description:
        "Preview changes before executing with --itemize-changes parsing, filtering, and virtualized display.",
      tag: "GUI" as const,
    },
    {
      icon: "code",
      title: "Command Parser",
      description:
        "Paste any rsync command to see what each flag does. Import commands as new jobs instantly.",
      tag: "Both" as const,
    },
    {
      icon: "archive",
      title: "Snapshot Retention",
      description:
        "Daily, weekly, and monthly retention policies automatically prune old snapshots.",
      tag: "Both" as const,
    },
    {
      icon: "monitor-small",
      title: "Terminal UI",
      description:
        "Full-featured TUI with vim keybindings, 4 themes, and headless execution for cron and systemd.",
      tag: "TUI" as const,
    },
    {
      icon: "download",
      title: "Export & Import",
      description:
        "Export statistics and job configurations. Import rsync commands as fully configured jobs.",
      tag: "Both" as const,
    },
    {
      icon: "palette",
      title: "Themes",
      description:
        "8-color theme system in the GUI with light/dark/system modes. 4 TUI themes: Default, Dark, Solarized, Nord.",
      tag: "Both" as const,
    },
  ],

  // Carousel settings
  carousel: {
    autoplay: {
      enabled: true,
      interval: 5000,
    },
    lightboxNavigation: true,
  },

  // Screenshots (starts empty — conditionally rendered)
  screenshots: [] as { src: string; alt: string; caption: string }[],

  // Architecture diagram
  architecture: {
    frontends: [
      {
        label: "Desktop GUI",
        description: "Tauri v2 + React + TypeScript + shadcn/ui",
        items: [
          "Visual job builder",
          "Live command preview",
          "Progress tracking",
          "System tray",
        ],
      },
      {
        label: "Terminal UI",
        description: "ratatui + crossterm + clap",
        items: [
          "Vim keybindings",
          "Headless execution",
          "4 color themes",
          "SSH-friendly",
        ],
      },
    ],
    core: {
      label: "Shared Rust Core",
      description: "rsync-core library",
      services: {
        label: "Services",
        items: [
          "Command builder & parser",
          "Job execution engine",
          "Statistics tracking",
          "NAS auto-detection",
          "Snapshot retention",
        ],
      },
      async: {
        label: "Background",
        items: [
          "Cron & interval scheduler",
          "History retention",
        ],
      },
    },
    persistence: {
      label: "Persistence",
      items: [
        "SQLite (rusqlite)",
        "Log files",
      ],
    },
  },

  // Getting started steps
  gettingStarted: {
    clone: {
      title: "Clone the Repo",
      steps: [
        {
          command: "git clone https://github.com/alleato-llc/rsync-studio.git",
          description: "Clone the repository",
        },
        { command: "cd rsync-desktop", description: "Enter the project directory" },
      ],
    },
    gui: {
      title: "Desktop GUI",
      steps: [
        { command: "npm install", description: "Install dependencies" },
        { command: "npm run tauri dev", description: "Launch with hot reload" },
      ],
    },
    tui: {
      title: "Terminal UI",
      steps: [
        {
          command: "cargo build -p rsync-commander --release",
          description: "Build the TUI binary",
        },
        {
          command: "./target/release/rsync-commander",
          description: "Launch interactive TUI",
        },
      ],
    },
  },

  // Download cards
  downloads: [
    {
      platform: "macOS",
      icon: "apple",
      label: "macOS",
      sublabel: ".dmg",
      href: "https://github.com/alleato-llc/rsync-studio/releases/latest",
    },
    {
      platform: "linux-deb",
      icon: "linux",
      label: "Linux",
      sublabel: ".deb (Debian/Ubuntu)",
      href: "https://github.com/alleato-llc/rsync-studio/releases/latest",
    },
    {
      platform: "linux-appimage",
      icon: "linux",
      label: "Linux",
      sublabel: "AppImage",
      href: "https://github.com/alleato-llc/rsync-studio/releases/latest",
    },
    {
      platform: "tui",
      icon: "terminal",
      label: "Terminal UI",
      sublabel: "cargo install",
      href: "https://github.com/alleato-llc/rsync-studio/releases/latest",
    },
  ],

  // Active development banner
  activeDevelopment: {
    enabled: true,
    message: "Open source and community-driven. Help us build the best rsync experience together.",
    links: {
      reportIssue: {
        enabled: true,
        url: "https://github.com/alleato-llc/rsync-studio/issues/new?template=bug_report.md",
        label: "Report a bug",
      },
      requestFeature: {
        enabled: true,
        url: "https://github.com/alleato-llc/rsync-studio/issues/new?template=feature_request.md",
        label: "Suggest a feature",
      },
      collaborate: {
        enabled: true,
        url: "https://github.com/alleato-llc/rsync-studio/pulls",
        label: "Contribute",
      },
    },
  },

  // Buy Me a Coffee
  buymeacoffee: {
    enabled: true,
    username: "nycjv321",
    url: "https://buymeacoffee.com/nycjv321",
    buttonText: "Buy me a coffee",
    message:
      "Rsync Studio is free, open source, and built in the open. If it saves you time, consider fueling the next feature.",
  },

  // Footer
  footer: {
    copyright: "Rsync Studio",
    year: new Date().getFullYear(),
    links: [
      {
        label: "GitHub",
        url: "https://github.com/alleato-llc/rsync-studio",
      },
      {
        label: "Documentation",
        url: "https://github.com/alleato-llc/rsync-studio/tree/main/docs",
      },
      {
        label: "Contributing",
        url: "https://github.com/alleato-llc/rsync-studio/blob/main/CONTRIBUTING.md",
      },
      {
        label: "License (MIT)",
        url: "https://github.com/alleato-llc/rsync-studio/blob/main/LICENSE",
      },
    ],
  },
};
