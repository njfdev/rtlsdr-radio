import AppView from "./components/AppView";
import { ThemeProvider } from "./components/ThemeProvider";

export default function App() {
  return (
    <ThemeProvider>
      <AppView />
    </ThemeProvider>
  );
}
