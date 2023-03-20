import ReactDOM from "react-dom/client";
import App from "./Pages/App";
import "./index.css";

import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import axios from "axios";
import { QueryClient, QueryClientProvider } from "react-query";
import TeamScore from "./Pages/TeamScore";
import AdminPage from "./Pages/Admin";
import TeamPasswords from "./Pages/TeamPasswords";

axios.defaults.baseURL = `http://${window.location.hostname}:8000/api`;
axios.defaults.headers.post["Content-Type"] = "application/json";

const queryClient = new QueryClient();

const LoginRouter = () => (
  <Routes>
    <Route path="/" element={<App />} />
    <Route path="/team/:teamName" element={<TeamScore />} />
    <Route path="/team/:teamName/passwords" element={<TeamPasswords />} />
    <Route path="/admin" element={<AdminPage />} />
  </Routes>
);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <QueryClientProvider client={queryClient}>
    <Router>
      <LoginRouter />
    </Router>
  </QueryClientProvider>
);
