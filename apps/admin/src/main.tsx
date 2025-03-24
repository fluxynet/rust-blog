import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import "./index.css";
import Layout from "./Layout";
import { ArticlesList, ArticleEdit, ArticleCreate } from "./Articles";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/articles" element={<ArticlesList />} />
          <Route path="/articles/new" element={<ArticleCreate />} />
          <Route path="/articles/:id" element={<ArticleEdit />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  </StrictMode>
);
