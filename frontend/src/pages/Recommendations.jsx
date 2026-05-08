import { useEffect, useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Recommendations() {
  const [recs, setRecs] = useState([])
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  useEffect(() => {
    api.get("/recommendations/")
      .then((res) => setRecs(res.data))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div>
      <h1>Recomendações</h1>
      <button onClick={() => navigate("/dashboard")}>Voltar</button>

      {loading && <p>Carregando...</p>}

      {!loading && recs.length === 0 && (
        <p>Nenhuma recomendação ainda. Adicione animes à sua lista primeiro.</p>
      )}

      {recs.map((anime) => (
        <div key={anime.anilist_id}>
          <img src={anime.cover_image_url} alt={anime.title_romaji} width={80} />
          <p>{anime.title_english || anime.title_romaji}</p>
          <p>{anime.genres?.join(", ")}</p>
          <p>{anime.episodes} episódios</p>
        </div>
      ))}
    </div>
  )
}
