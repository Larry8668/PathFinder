import { useEffect, useState } from "react";

export function useFirstLaunch() {
  const [isFirstLaunch, setIsFirstLaunch] = useState(null);

  useEffect(() => {
    const flag = localStorage.getItem("firstLaunch");

    if (flag === null) {
      localStorage.setItem("firstLaunch", "true");
      setIsFirstLaunch(true);
    } else {
      setIsFirstLaunch(false);
    }
  }, []);

  return isFirstLaunch;
}
