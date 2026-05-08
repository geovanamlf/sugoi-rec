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
    <div className="min-h-screen">
      <div className="page-container">

        <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "2rem" }}>
          <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
          <h1 className="pixel-title" style={{ fontSize: "36px" }}>RECOMENDAÇÕES</h1>
        </div>

        {loading && (
          <div className="pixel-box" style={{ maxWidth: "300px" }}>
            <p className="font-pixel" style={{ fontSize: "20px", color: "#c9a87c" }}>⌛ carregando...</p>
          </div>
        )}

        {!loading && recs.length === 0 && (
          <div className="pixel-box" style={{ maxWidth: "400px" }}>
            <p style={{ fontSize: "14px", color: "#a8a8c0" }}>
              nenhuma recomendação ainda.<br /><br />
              adicione animes à sua lista primeiro!
            </p>
          </div>
        )}

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: "1rem" }}>
          {recs.map((anime) => (
            <div key={anime.anilist_id} className="pixel-box" style={{ padding: "0.75rem", display: "flex", flexDirection: "column", gap: "0.5rem" }}>
              {anime.cover_image_url && (
                <img src={anime.cover_image_url} alt={anime.title_romaji}
                  style={{ width: "100%", height: "180px", objectFit: "cover", border: "3px solid #e8d5b7" }} />
              )}
              <h2 className="font-pixel" style={{ fontSize: "16px", color: "#a8c5a0", lineHeight: "1.4" }}>
                {anime.title_english || anime.title_romaji}
              </h2>
              <p style={{ fontSize: "12px", color: "#c9a87c" }}>
                {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
              </p>
              <p style={{ fontSize: "12px", color: "#a8a8c0" }}>
                {anime.genres?.join(" · ")}
              </p>
            </div>
          ))}
        </div>

      </div>
    </div>
  )
}