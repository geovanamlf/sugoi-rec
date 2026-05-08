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
    <div className="min-h-screen p-4 md:p-8 max-w-6xl mx-auto">

      {/* Header */}
      <div className="flex flex-col sm:flex-row items-center justify-between mb-8 gap-4">
        <h1 className="pixel-title">SUGOI REC</h1>
        <div className="flex gap-3 flex-wrap justify-center">
          <button className="pixel-btn" onClick={() => navigate("/search")}>» BUSCAR</button>
          <button className="pixel-btn" onClick={() => navigate("/recommendations")}>★ RECS</button>
          <button className="pixel-btn pixel-btn-danger" onClick={handleLogout}>SAIR</button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        <div className="pixel-box">
          <h2 className="pixel-subtitle">📊 estatísticas</h2>
          {stats && (
            <div className="flex flex-col gap-2 text-sm">
              <p>média de rating: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{stats.average_rating ?? "—"}</span></p>
              <p>animes avaliados: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{stats.rated_count}</span></p>
              <p>total na lista: <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{list.length}</span></p>
            </div>
          )}
        </div>

        <div className="pixel-box">
          <h2 className="pixel-subtitle">🎭 top gêneros</h2>
          <div className="flex flex-col gap-2">
            {genres.map((g) => (
              <div key={g.genre} className="flex justify-between text-sm">
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
          <p className="text-sm" style={{ color: "#a8a8c0" }}>
            nenhum anime ainda. busque um para adicionar!
          </p>
        )}

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {list.map((item) => (
            <div key={item.id} className="p-4 flex flex-col gap-2"
              style={{ border: "2px solid #3d3d5a", background: "#1a1a2e" }}>
              <p className="font-pixel text-xl" style={{ color: "#e8d5b7" }}>anime #{item.anime_id}</p>
              <p className="text-sm font-semibold" style={{ color: statusColor[item.status] }}>
                {statusLabel[item.status]}
              </p>
              <p className="text-sm" style={{ color: "#c9a87c" }}>
                nota: {item.rating ?? "—"} {item.is_favorite ? "♥" : ""}
              </p>
            </div>
          ))}
        </div>
      </div>

    </div>
  )
}