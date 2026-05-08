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
  const [filter, setFilter] = useState("all")
  const [editing, setEditing] = useState(null)

  useEffect(() => {
    loadData()
  }, [])

  function loadData() {
    api.get("/list/").then((res) => setList(res.data))
    api.get("/analytics/ratings").then((res) => setStats(res.data))
    api.get("/analytics/genres").then((res) => setGenres(res.data.slice(0, 5)))
  }

  function handleLogout() {
    logout()
    navigate("/login")
  }

  async function handleRemove(animeId) {
    await api.delete(`/list/${animeId}`)
    loadData()
  }

  async function handleUpdate(animeId, data) {
    await api.patch(`/list/${animeId}`, data)
    setEditing(null)
    loadData()
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

  const filters = [
    { key: "all", label: "TODOS" },
    { key: "watching", label: "▶ ASSISTINDO" },
    { key: "completed", label: "✓ COMPLETO" },
    { key: "dropped", label: "✗ DROPADO" },
    { key: "planned", label: "◎ PLANEJADO" },
  ]

  const filteredList = filter === "all" ? list : list.filter((i) => i.status === filter)

  const counts = {
    watching: list.filter((i) => i.status === "watching").length,
    completed: list.filter((i) => i.status === "completed").length,
    dropped: list.filter((i) => i.status === "dropped").length,
    planned: list.filter((i) => i.status === "planned").length,
  }

  return (
    <div className="min-h-screen">
      <div className="page-container">

        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "1rem", marginBottom: "0.5rem" }}>
          <h1 className="pixel-title">SUGOI REC</h1>
          <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
            <button className="pixel-btn" onClick={() => navigate("/search")}>» BUSCAR</button>
            <button className="pixel-btn" onClick={() => navigate("/recommendations")}>★ RECS</button>
            <button className="pixel-btn pixel-btn-danger" onClick={handleLogout}>SAIR</button>
          </div>
        </div>

        {/* Status count */}
        <p style={{ fontSize: "13px", color: "#a8a8c0", marginBottom: "2rem" }}>
          <span style={{ color: "#a8c5a0" }}>▶ {counts.watching}</span>
          {" · "}
          <span style={{ color: "#7ab8d4" }}>✓ {counts.completed}</span>
          {" · "}
          <span style={{ color: "#e07070" }}>✗ {counts.dropped}</span>
          {" · "}
          <span style={{ color: "#c9a87c" }}>◎ {counts.planned}</span>
        </p>

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

          {/* Filtros */}
          <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap", marginBottom: "1.5rem" }}>
            {filters.map((f) => (
              <button
                key={f.key}
                onClick={() => setFilter(f.key)}
                className="pixel-btn"
                style={{
                  fontSize: "13px",
                  padding: "6px 12px",
                  backgroundColor: filter === f.key ? "#5a3e7a" : undefined,
                  borderColor: filter === f.key ? "#c9a8f0" : undefined,
                }}
              >
                {f.label}
                {f.key !== "all" && (
                  <span style={{ color: "#a8a8c0", marginLeft: "6px" }}>({counts[f.key]})</span>
                )}
              </button>
            ))}
          </div>

          {filteredList.length === 0 && (
            <p style={{ fontSize: "14px", color: "#a8a8c0" }}>
              {filter === "all" ? "nenhum anime ainda. busque um para adicionar!" : `nenhum anime com status "${filter}".`}
            </p>
          )}

          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: "1rem" }}>
            {filteredList.map((item) => (
              <div
                key={item.id}
                style={{ border: "2px solid #3d3d5a", background: "#1a1a2e", overflow: "hidden", cursor: "pointer" }}
                onClick={() => navigate(`/anime/${item.anime?.anilist_id}`)}
              >
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
                  <div style={{ display: "flex", gap: "0.5rem", marginTop: "0.5rem" }}>
                    <button
                      className="pixel-btn"
                      style={{ fontSize: "12px", padding: "4px 8px", flex: 1 }}
                      onClick={(e) => { e.stopPropagation(); setEditing(item) }}
                    >
                      ✎ editar
                    </button>
                    <button
                      className="pixel-btn pixel-btn-danger"
                      style={{ fontSize: "12px", padding: "4px 8px" }}
                      onClick={(e) => { e.stopPropagation(); handleRemove(item.anime_id) }}
                    >
                      ✕
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

      </div>

      {/* Modal de edição */}
      {editing && (
        <EditModal
          item={editing}
          onClose={() => setEditing(null)}
          onSave={handleUpdate}
        />
      )}
    </div>
  )
}

function EditModal({ item, onClose, onSave }) {
  const [status, setStatus] = useState(item.status)
  const [rating, setRating] = useState(item.rating ?? "")
  const [favorite, setFavorite] = useState(item.is_favorite)

  return (
    <div style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.7)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 50, padding: "1rem"
    }}>
      <div className="pixel-box" style={{ width: "100%", maxWidth: "400px" }}>
        <h2 className="pixel-subtitle">✎ editar</h2>
        <p className="font-pixel" style={{ fontSize: "14px", color: "#a8c5a0", marginBottom: "1rem", lineHeight: "1.4" }}>
          {item.anime?.title_english || item.anime?.title_romaji}
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
          <div>
            <p style={{ fontSize: "14px", color: "#c9a87c", marginBottom: "0.5rem" }}>status:</p>
            <select className="pixel-input" value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="planned">planejado</option>
              <option value="watching">assistindo</option>
              <option value="completed">completado</option>
              <option value="dropped">dropado</option>
            </select>
          </div>

          <div>
            <p style={{ fontSize: "14px", color: "#c9a87c", marginBottom: "0.5rem" }}>nota (1-10):</p>
            <input
              className="pixel-input"
              type="number"
              placeholder="opcional"
              value={rating}
              onChange={(e) => setRating(e.target.value)}
              min={1} max={10}
            />
          </div>

          <label style={{ display: "flex", alignItems: "center", gap: "0.75rem", fontSize: "14px", cursor: "pointer" }}>
            <input type="checkbox" checked={favorite} onChange={(e) => setFavorite(e.target.checked)} />
            <span style={{ color: "#e8d5b7" }}>♥ favorito</span>
          </label>

          <div style={{ display: "flex", gap: "0.75rem" }}>
            <button
              className="pixel-btn"
              style={{ flex: 1 }}
              onClick={() => onSave(item.anime_id, {
                status,
                rating: rating ? parseInt(rating) : null,
                is_favorite: favorite,
              })}
            >
              ✓ salvar
            </button>
            <button className="pixel-btn pixel-btn-danger" onClick={onClose}>
              ✕ cancelar
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}