import ReactDOM from "react-dom/client";
import App from "./Pages/App";
import "./index.css";

import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import axios from "axios";
import { QueryClient, QueryClientProvider } from "react-query";
import TeamScore from "./Pages/TeamScore";
import AdminPage from "./Pages/Admin";
import TeamInject from "./Pages/TeamInject";
import TeamPasswords from "./Pages/TeamPasswords";
import LoginPage from "./Pages/Login";
import { useScoreboardInfo } from "./Hooks/CtrlHooks";

if (import.meta.env.PROD) {
  axios.defaults.baseURL = `http://${window.location.hostname}:${window.location.port}/api`;
} else {
  axios.defaults.baseURL = `http://${window.location.hostname}:8000/api`;
}
axios.defaults.headers.post["Content-Type"] = "application/json";
axios.defaults.withCredentials = true

const queryClient = new QueryClient();

const LoginRouter = () => {
  const { info, infoLoading } = useScoreboardInfo()
  return (
    <main>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/team/:teamName" element={<TeamScore />} />
        <Route path="/team/:teamName/passwords" element={<TeamPasswords />} />
        <Route path="/team/:teamName/inject/:injectId" element={<TeamInject />} />
        <Route path="/admin" element={<AdminPage />} />
        <Route path="/login" element={<LoginPage />} />
      </Routes>
      {!infoLoading && <div className="fixed text-sm right-2 bottom-1 font-light">Scoreboard v{info.version}</div>}
    </main>
  )
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  <QueryClientProvider client={queryClient}>
    <Router>
      <LoginRouter />
    </Router>
  </QueryClientProvider>
);
