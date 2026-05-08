import { useEffect, useState } from "react"
import { useParams, useNavigate } from "react-router-dom"
import api from "../api/client"

export default function AnimeDetail() {
  const { animeId } = useParams()
  const navigate = useNavigate()
  const [anime, setAnime] = useState(null)
  const [error, setError] = useState(null)

  useEffect(() => {
    api.get(`/anime/id/${animeId}`)
      .then((res) => setAnime(res.data))
      .catch(() => setError("anime não encontrado."))
  }, [animeId])

  if (error) return (
    <div className="min-h-screen">
      <div className="page-container">
        <button className="pixel-btn" onClick={() => navigate(-1)}>« VOLTAR</button>
        <p style={{ marginTop: "2rem", color: "#e07070" }}>✗ {error}</p>
      </div>
    </div>
  )

  if (!anime) return (
    <div className="min-h-screen">
      <div className="page-container">
        <p className="font-pixel" style={{ color: "#c9a87c", fontSize: "20px" }}>⌛ carregando...</p>
      </div>
    </div>
  )

  return (
    <div className="min-h-screen">
      <div className="page-container">

        <button className="pixel-btn" style={{ marginBottom: "2rem" }} onClick={() => navigate(-1)}>
          « VOLTAR
        </button>

        <div style={{ display: "flex", gap: "2rem", flexWrap: "wrap", marginBottom: "2rem" }}>
          {anime.cover_image_url && (
            <img
              src={anime.cover_image_url}
              alt={anime.title_romaji}
              style={{ width: "180px", border: "4px solid #e8d5b7", boxShadow: "6px 6px 0 #000", alignSelf: "flex-start" }}
            />
          )}
          <div style={{ flex: 1, minWidth: "200px" }}>
            <h1 className="pixel-title" style={{ fontSize: "28px", marginBottom: "0.5rem" }}>
              {anime.title_english || anime.title_romaji}
            </h1>
            {anime.title_english && (
              <p className="font-pixel" style={{ fontSize: "14px", color: "#a8a8c0", marginBottom: "1rem" }}>
                {anime.title_romaji}
              </p>
            )}
            {anime.title_native && (
              <p style={{ fontSize: "14px", color: "#a8a8c0", marginBottom: "1rem" }}>
                {anime.title_native}
              </p>
            )}
            <p style={{ fontSize: "14px", color: "#c9a87c", marginBottom: "0.5rem" }}>
              {anime.episode_count ? `${anime.episode_count} episódios` : "episódios: —"}
            </p>
            {anime.demographic && (
              <p style={{ fontSize: "14px", color: "#a8a8c0", marginBottom: "0.5rem" }}>
                {anime.demographic}
              </p>
            )}
          </div>
        </div>

        {/* Gêneros */}
        {anime.genres && (
          <div className="pixel-box" style={{ marginBottom: "1.5rem" }}>
            <h2 className="pixel-subtitle">🎭 gêneros</h2>
            <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
              {anime.genres.split(",").map((g) => (
                <span key={g} style={{
                  fontSize: "13px", padding: "4px 10px",
                  border: "2px solid #a8c5a0", color: "#a8c5a0",
                  background: "#1a1a2e"
                }}>
                  {g.trim()}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Descrição */}
        {anime.description && (
          <div className="pixel-box" style={{ marginBottom: "1.5rem" }}>
            <h2 className="pixel-subtitle">📖 descrição</h2>
            <p style={{ fontSize: "14px", lineHeight: "1.8", color: "#c8bfa8" }}>
              {anime.description.replace(/<[^>]+>/g, "")}
            </p>
          </div>
        )}

        {/* Tags */}
        {anime.tags && (
          <div className="pixel-box">
            <h2 className="pixel-subtitle">🏷️ tags</h2>
            <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
              {anime.tags.split(",").map((t) => (
                <span key={t} style={{
                  fontSize: "12px", padding: "3px 8px",
                  border: "2px solid #3d3d5a", color: "#a8a8c0",
                  background: "#1a1a2e"
                }}>
                  {t.trim()}
                </span>
              ))}
            </div>
          </div>
        )}

      </div>
    </div>
  )
}
