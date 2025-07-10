import Fuse from "fuse.js";

const dummyClips = [
  { text: "Hello world" },
  { text: "Important email copied" },
  { text: "Some other copied text" },
];

const fuse = new Fuse(dummyClips, { keys: ["text"], threshold: 0.4 });

export default function ClipboardPage({ query }) {
  const filtered = query
    ? fuse.search(query).map((res) => res.item)
    : dummyClips;

  return (
    <div className="option-list">
      {filtered.map((clip, idx) => (
        <div className="option-item" key={idx}>
          <span className="icon">📄</span>
          <span>{clip.text}</span>
        </div>
      ))}
    </div>
  );
}
