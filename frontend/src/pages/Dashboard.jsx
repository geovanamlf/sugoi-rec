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

  return (
    <div>
      <h1>Dashboard</h1>
      <button onClick={handleLogout}>Sair</button>
      <button onClick={() => navigate("/search")}>Buscar Anime</button>
      <button onClick={() => navigate("/recommendations")}>Recomendações</button>

      <h2>Minha Lista ({list.length} animes)</h2>
      {list.length === 0 && <p>Nenhum anime na lista ainda.</p>}
      {list.map((item) => (
        <div key={item.id}>
          <p>Anime ID: {item.anime_id} — Status: {item.status} — Nota: {item.rating ?? "sem nota"}</p>
        </div>
      ))}

      <h2>Estatísticas</h2>
      {stats && (
        <p>Média de rating: {stats.average_rating ?? "nenhuma nota ainda"} ({stats.rated_count} avaliados)</p>
      )}

      <h2>Top Gêneros</h2>
      {genres.map((g) => (
        <p key={g.genre}>{g.genre}: {g.count}</p>
      ))}
    </div>
  )
}
