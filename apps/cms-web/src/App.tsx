import { Navigate, Route, Routes } from "react-router-dom";
import { AuthProvider } from "./auth/AuthProvider";
import { ProtectedRoute, PublicOnlyRoute } from "./auth/RouteGuards";
import { AppShell } from "./layout/AppShell";
import { LoginPage } from "./pages/LoginPage";
import { DashboardPage } from "./pages/DashboardPage";
import { HomeEditorPage } from "./pages/HomeEditorPage";
import { ContentListPage } from "./pages/ContentListPage";
import { ContentFormPage } from "./pages/ContentFormPage";
import { MediaLibraryPage } from "./pages/MediaLibraryPage";
import { PreviewPage } from "./pages/PreviewPage";

export function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route
          path="/login"
          element={
            <PublicOnlyRoute>
              <LoginPage />
            </PublicOnlyRoute>
          }
        />
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <AppShell />
            </ProtectedRoute>
          }
        >
          <Route index element={<DashboardPage />} />
          <Route path="home" element={<HomeEditorPage />} />
          <Route path="content" element={<ContentListPage />} />
          <Route path="content/new" element={<ContentFormPage />} />
          <Route path="content/:id" element={<ContentFormPage />} />
          <Route path="media" element={<MediaLibraryPage />} />
          <Route path="preview" element={<PreviewPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AuthProvider>
  );
}
