export default function OnlineSearchPage({ query }) {
  return (
    <div className="option-list">
      <div className="option-item">
        <span className="icon">🌐</span>
        <span>Search online for “{query}”</span>
      </div>
    </div>
  );
}
