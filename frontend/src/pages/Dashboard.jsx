import { useEffect, useState } from "react"
import { useAuth } from "../context/AuthContext"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Dashboard() {
  const { logout } = useAuth()
  const navigate = useNavigate()
  const [list, setList] = useState([])
  const [stats, setStats] = useState(null)
  const [genres, setGenres] = useState([])

  useEffect(() => {
    api.get("/list/").then((res) => setList(res.data))
    api.get("/analytics/ratings").then((res) => setStats(res.data))
    api.get("/analytics/genres").then((res) => setGenres(res.data.slice(0, 5)))
  }, [])

  function handleLogout() {
    logout()
    navigate("/login")
  }

  const statusLabel = {
    watching: "▶ assistindo",
    completed: "✓ completo",
    dropped: "✗ dropado",
    planned: "◎ planejado",
  }

  const statusColor = {
    watching: "#a8c5a0",
    completed: "#7ab8d4",
    dropped: "#e07070",
    planned: "#c9a87c",
  }

  return (
    <div className="min-h-screen">
      <div className="page-container">

        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "1rem", marginBottom: "2rem" }}>
          <h1 className="pixel-title">SUGOI REC</h1>
          <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
            <button className="pixel-btn" onClick={() => navigate("/search")}>» BUSCAR</button>
            <button className="pixel-btn" onClick={() => navigate("/recommendations")}>★ RECS</button>
            <button className="pixel-btn pixel-btn-danger" onClick={handleLogout}>SAIR</button>
          </div>
        </div>

        {/* Stats */}
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "1.5rem", marginBottom: "1.5rem" }}>
          <div className="pixel-box">
            <h2 className="pixel-subtitle">📊 estatísticas</h2>
            {stats && (
              <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", fontSize: "14px" }}>
                <p>média de rating: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{stats.average_rating ?? "—"}</span></p>
                <p>animes avaliados: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{stats.rated_count}</span></p>
                <p>total na lista: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{list.length}</span></p>
              </div>
            )}
          </div>

          <div className="pixel-box">
            <h2 className="pixel-subtitle">🎭 top gêneros</h2>
            <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
              {genres.map((g) => (
                <div key={g.genre} style={{ display: "flex", justifyContent: "space-between", fontSize: "14px" }}>
                  <span>{g.genre}</span>
                  <span style={{ color: "#c9a87c", fontWeight: 700 }}>{g.count}x</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Lista */}
        <div className="pixel-box">
          <h2 className="pixel-subtitle">📋 minha lista ({list.length})</h2>

          {list.length === 0 && (
            <p style={{ fontSize: "14px", color: "#a8a8c0" }}>
              nenhum anime ainda. busque um para adicionar!
            </p>
          )}

          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: "1rem" }}>
            {list.map((item) => (
              <div key={item.id} style={{ border: "2px solid #3d3d5a", background: "#1a1a2e", overflow: "hidden" }}>
                {/* Capa do anime */}
                {item.anime?.cover_image_url && (
                  <img
                    src={item.anime.cover_image_url}
                    alt={item.anime.title_romaji}
                    style={{ width: "100%", height: "140px", objectFit: "cover", borderBottom: "2px solid #3d3d5a" }}
                  />
                )}
                <div style={{ padding: "0.75rem", display: "flex", flexDirection: "column", gap: "0.4rem" }}>
                  <p className="font-pixel" style={{ fontSize: "14px", color: "#e8d5b7", lineHeight: "1.4" }}>
                    {item.anime?.title_english || item.anime?.title_romaji || `anime #${item.anime_id}`}
                  </p>
                  <p style={{ fontSize: "13px", fontWeight: 600, color: statusColor[item.status] }}>
                    {statusLabel[item.status]}
                  </p>
                  <p style={{ fontSize: "13px", color: "#c9a87c" }}>
                    nota: {item.rating ?? "—"} {item.is_favorite ? "♥" : ""}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>

      </div>
    </div>
  )
}