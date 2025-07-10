import { useState, useEffect, useCallback } from 'react';

export function useKeyboardNavigation(items, onEnter, initialIndex = 0) {
  const [selectedIndex, setSelectedIndex] = useState(initialIndex);
  const itemCount = items?.length || 0;

  // Reset selected index when items change
  useEffect(() => {
    setSelectedIndex(0);
  }, [items]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e) => {
    if (!itemCount) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => (prev + 1) % itemCount);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => (prev - 1 + itemCount) % itemCount);
        break;
      case 'Enter':
        if (items[selectedIndex]) {
          e.preventDefault();
          onEnter?.(items[selectedIndex], selectedIndex);
        }
        break;
      default:
        break;
    }
  }, [itemCount, items, onEnter, selectedIndex]);

  // Add and cleanup event listeners
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return {
    selectedIndex,
    setSelectedIndex,
    getItemProps: (index) => ({
      className: `option-item ${selectedIndex === index ? 'selected' : ''}`,
      onClick: () => onEnter?.(items[index], index)
    })
  };
}
