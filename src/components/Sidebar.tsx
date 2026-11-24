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
  function handleKeyDown(event: React.KeyboardEvent<HTMLElement>, index: number) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      const nextIndex = (index + 1) % navigation.length;
      const nextBtn = document.getElementById(`nav-btn-${navigation[nextIndex].id}`);
      nextBtn?.focus();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      const prevIndex = (index - 1 + navigation.length) % navigation.length;
      const prevBtn = document.getElementById(`nav-btn-${navigation[prevIndex].id}`);
      prevBtn?.focus();
    }
  }

  return (
    <aside className="sidebar" aria-label="Bilah navigasi aplikasi">
      <div className="brand">
        <div className="brand-mark" aria-hidden="true">S</div>
        <div>
          <strong>Survey Copilot</strong>
          <span>Asisten survei AI</span>
        </div>
      </div>

      <nav aria-label="Navigasi utama">
        {navigation.map((item, index) => (
          <button
            key={item.id}
            id={`nav-btn-${item.id}`}
            className={activePage === item.id ? "nav-item active" : "nav-item"}
            aria-current={activePage === item.id ? "page" : undefined}
            aria-label={`${item.label} (Halaman ${item.label})`}
            onClick={() => onSelectPage(item.id)}
            onKeyDown={(e) => handleKeyDown(e, index)}
          >
            <span className="nav-icon" aria-hidden="true">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </nav>

      <div className="sidebar-bottom">
        <span className="status-dot" aria-hidden="true" title="Aplikasi beroperasi secara lokal" />
        <span>Mode lokal</span>
        <button
          className={activePage === "settings" ? "settings-link active" : "settings-link"}
          aria-current={activePage === "settings" ? "page" : undefined}
          aria-label="Pengaturan aplikasi"
          onClick={() => onSelectPage("settings")}
        >
          ⚙️ Pengaturan
        </button>
      </div>
    </aside>
  );
}
