import Fuse from "fuse.js";

const dummyFiles = [
  { text: "Document.pdf" },
  { text: "Resume.docx" },
  { text: "Presentation.pptx" },
];

const fuse = new Fuse(dummyFiles, { keys: ["text"], threshold: 0.4 });

export default function OpenFilePage({ query }) {
  const filteredFiles = query
    ? fuse.search(query).map((res) => res.item)
    : dummyFiles;

  return (
    <div className="option-list">
      {filteredFiles.map((file, idx) => (
        <div className="option-item" key={idx}>
          <span className="icon">📁</span>
          <span>{file.text}</span>
        </div>
      ))}
    </div>
  );
}
