export default function ClipboardPage({ query }) {
  return (
    <div className="page clipboard-page">
      <p>
        Showing clipboard results for: <strong>{query}</strong>
      </p>
    </div>
  );
}
