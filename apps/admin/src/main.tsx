import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import "./index.css";
// import LoginPage from "./Login";
import Layout from "./Layout";
import { ArticlesList } from "./Articles";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      {/* <LoginPage /> */}
      <Layout>
        <Routes>
          <Route path="/articles" element={<ArticlesList />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  </StrictMode>
);
