const OPTIONS = [
  { title: "Clipboard", icon: "📋" },
  { title: "Online Search", icon: "🔍" },
  { title: "Open File", icon: "📁" },
];

export default function HomeOptions({ query }) {
  const filtered = OPTIONS.filter((opt) =>
    opt.title.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div className="option-list">
      {filtered.length ? (
        filtered.map((opt, idx) => (
          <div className="option-item" key={idx}>
            <span className="icon">{opt.icon}</span>
            <span>{opt.title}</span>
          </div>
        ))
      ) : (
        <div className="option-item">
          <span className="icon">🌐</span>
          <span>Search online for “{query}”</span>
        </div>
      )}
    </div>
  );
}
