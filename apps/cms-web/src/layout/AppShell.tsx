import { NavLink, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthProvider";
import "../styles.css";

export function AppShell() {
  const { admin, logout } = useAuth();

  return (
    <div className="cms-layout">
      <aside className="cms-sidebar">
        <div className="cms-sidebar-brand">Civico</div>
        <nav className="cms-nav">
          <NavLink to="/" className="cms-nav-link" end>
            Dashboard
          </NavLink>
          <NavLink to="/home" className="cms-nav-link">
            Home
          </NavLink>
          <NavLink to="/content" className="cms-nav-link">
            Contenuti
          </NavLink>
          <NavLink to="/media" className="cms-nav-link">
            Media
          </NavLink>
          <NavLink to="/preview" className="cms-nav-link">
            Anteprima
          </NavLink>
        </nav>
        <div className="cms-sidebar-footer">
          <div className="cms-admin-info">{admin?.username ?? "Admin"}</div>
          <button type="button" className="cms-logout-button" onClick={logout}>
            Esci
          </button>
        </div>
      </aside>
      <main className="cms-main">
        <Outlet />
      </main>
    </div>
  );
}
