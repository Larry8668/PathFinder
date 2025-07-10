import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";

export default function OnlineSearchPage({ query }) {
  const { getItemProps } = useKeyboardNavigation([query], (item, idx) => {
    console.log("Selected:", item);
  });

  return (
    <div className="option-list">
      <div
        {...getItemProps(0)}
        className={`option-item ${getItemProps(0).className}`}
      >
        <span className="icon">🌐</span>
        <span>Search online for “{query}”</span>
      </div>
    </div>
  );
}
