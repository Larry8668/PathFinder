import Fuse from "fuse.js";
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";

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

  const { getItemProps } = useKeyboardNavigation(filteredFiles, (item, idx) => {
    console.log("Selected:", item.text);
  });

  return (
    <div className="option-list">
      {filteredFiles.map((file, idx) => (
        <div
          {...getItemProps(idx)}
          className={`option-item ${getItemProps(idx).className}`}
          key={idx}
        >
          <span className="icon">📁</span>
          <span>{file.text}</span>
        </div>
      ))}
    </div>
  );
}
