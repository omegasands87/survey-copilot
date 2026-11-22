export type Page = "home" | "profiles" | "copilot" | "history" | "settings";

interface SidebarProps {
  activePage: Page;
  onSelectPage: (page: Page) => void;
}

const navigation: { id: Page; label: string; icon: string }[] = [
  { id: "home", label: "Beranda", icon: "⌂" },
  { id: "profiles", label: "Profil", icon: "◉" },
  { id: "copilot", label: "Copilot", icon: "✳" },
  { id: "history", label: "Riwayat", icon: "◷" },
];

export function Sidebar({ activePage, onSelectPage }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">S</div>
        <div>
          <strong>Survey Copilot</strong>
          <span>Asisten survei AI</span>
        </div>
      </div>

      <nav aria-label="Navigasi utama">
        {navigation.map((item) => (
          <button
            key={item.id}
            className={activePage === item.id ? "nav-item active" : "nav-item"}
            onClick={() => onSelectPage(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </nav>

      <div className="sidebar-bottom">
        <span className="status-dot" />
        <span>Mode lokal</span>
        <button className="settings-link" onClick={() => onSelectPage("settings")}>
          Pengaturan
        </button>
      </div>
    </aside>
  );
}
