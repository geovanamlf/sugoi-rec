import { useEffect, useMemo, useRef, useState } from "react"
import { useNavigate } from "react-router-dom"
import api from "../api/client"

export default function Recommendations() {
  const [recs, setRecs] = useState([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [adding, setAdding] = useState(null)
  const [activeGenre, setActiveGenre] = useState("TODOS")
  const [error, setError] = useState(null)

  const requestIdRef = useRef(0)
  const navigate = useNavigate()

  function loadRecs(refresh = false, signal) {
    const requestId = requestIdRef.current + 1
    requestIdRef.current = requestId

    setError(null)

    if (refresh) {
      setRefreshing(true)
    } else if (recs.length === 0) {
      setLoading(true)
    }

    api.get(`/recommendations/?refresh=${refresh}`, { signal })
      .then((res) => {
        if (requestId !== requestIdRef.current) return

        setRecs(res.data)
      })
      .catch((err) => {
        if (requestId !== requestIdRef.current) return

        if (err?.code === "ERR_CANCELED" || err?.name === "CanceledError") {
          return
        }

        const status = err?.response?.status
        const detail = err?.response?.data?.detail

        console.error("Erro ao carregar recomendações:", status, err?.response?.data || err)

        if (status === 429) {
          setError("A AniList limitou as buscas por enquanto. Aguarde um pouco ou use recomendações já carregadas.")
          return
        }

        if (status === 401) {
          setError("Sua sessão expirou. Faça login novamente.")
          return
        }

        if (status === 503) {
          setError("A AniList está instável agora. Tente novamente em alguns minutos.")
          return
        }

        setError(detail || "Não foi possível carregar recomendações agora.")
      })
      .finally(() => {
        if (requestId !== requestIdRef.current) return

        setLoading(false)
        setRefreshing(false)
      })
  }

  useEffect(() => {
    const controller = new AbortController()

    loadRecs(false, controller.signal)

    return () => {
      controller.abort()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const allGenres = useMemo(() => {
    const genres = new Set()
    recs.forEach((anime) => anime.genres?.forEach((g) => genres.add(g)))
    return ["TODOS", ...Array.from(genres).sort()]
  }, [recs])

  const filtered = activeGenre === "TODOS"
    ? recs
    : recs.filter((anime) => anime.genres?.includes(activeGenre))

  function handleRecommendationAdded(anilistId) {
    setRecs((current) => current.filter((anime) => anime.anilist_id !== anilistId))
  }

  return (
    <div className="min-h-screen">
      <div className="page-container">

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "1rem", marginBottom: "2rem" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "1rem" }}>
            <button className="pixel-btn" onClick={() => navigate("/dashboard")}>« VOLTAR</button>
            <h1 className="pixel-title" style={{ fontSize: "36px" }}>RECOMENDAÇÕES</h1>
          </div>
          <button
            className="pixel-btn"
            style={{ fontSize: "13px", opacity: refreshing ? 0.6 : 1 }}
            onClick={() => loadRecs(true)}
            disabled={refreshing || loading}
          >
            {refreshing ? "⌛ atualizando..." : "↺ atualizar"}
          </button>
        </div>

        {loading && (
          <div className="pixel-box" style={{ maxWidth: "300px" }}>
            <p className="font-pixel" style={{ fontSize: "20px", color: "#c9a87c" }}>⌛ carregando...</p>
          </div>
        )}

        {!loading && error && (
          <div className="pixel-box" style={{ maxWidth: "520px", marginBottom: "1.5rem" }}>
            <p style={{ fontSize: "14px", color: "#e07070", lineHeight: "1.7" }}>
              {error}
            </p>
            {recs.length > 0 && (
              <p style={{ fontSize: "12px", color: "#a8a8c0", marginTop: "0.75rem" }}>
                mantendo as recomendações que já estavam carregadas.
              </p>
            )}
          </div>
        )}

        {!loading && !error && recs.length === 0 && (
          <div className="pixel-box" style={{ maxWidth: "400px" }}>
            <p style={{ fontSize: "14px", color: "#a8a8c0" }}>
              nenhuma recomendação ainda.<br /><br />
              adicione animes à sua lista primeiro!
            </p>
          </div>
        )}

        {!loading && recs.length > 0 && (
          <>
            <div className="pixel-box" style={{ marginBottom: "1.5rem" }}>
              <h2 className="pixel-subtitle">🎭 filtrar por gênero</h2>
              <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
                {allGenres.map((genre) => (
                  <button
                    key={genre}
                    className="pixel-btn"
                    style={{
                      fontSize: "12px",
                      padding: "4px 10px",
                      backgroundColor: activeGenre === genre ? "#5a3e7a" : undefined,
                      borderColor: activeGenre === genre ? "#c9a8f0" : undefined,
                    }}
                    onClick={() => setActiveGenre(genre)}
                  >
                    {genre}
                  </button>
                ))}
              </div>
            </div>

            <p style={{ fontSize: "13px", color: "#a8a8c0", marginBottom: "1rem" }}>
              mostrando <span style={{ color: "#a8c5a0", fontWeight: 700 }}>{filtered.length}</span> de {recs.length} recomendações
            </p>

            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: "1rem" }}>
              {filtered.map((anime) => (
                <div
                  key={anime.anilist_id}
                  className="pixel-box"
                  style={{ padding: "0.75rem", display: "flex", flexDirection: "column", gap: "0.5rem", cursor: "pointer" }}
                  onClick={() => navigate(`/anime/${anime.anilist_id}`)}
                >
                  {anime.cover_image_url && (
                    <img
                      src={anime.cover_image_url}
                      alt={anime.title_romaji}
                      style={{ width: "100%", height: "180px", objectFit: "cover", border: "3px solid #e8d5b7" }}
                    />
                  )}
                  <h2 className="font-pixel" style={{ fontSize: "14px", color: "#a8c5a0", lineHeight: "1.4" }}>
                    {anime.title_english || anime.title_romaji}
                  </h2>
                  <p style={{ fontSize: "12px", color: "#c9a87c" }}>
                    {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
                  </p>
                  <p style={{ fontSize: "12px", color: "#a8a8c0" }}>
                    {anime.genres?.join(" · ")}
                  </p>
                  <button
                    className="pixel-btn"
                    style={{ fontSize: "13px", padding: "6px", marginTop: "auto" }}
                    onClick={(e) => { e.stopPropagation(); setAdding(anime) }}
                  >
                    + ADICIONAR
                  </button>
                </div>
              ))}
            </div>
          </>
        )}

      </div>

      {adding && (
        <AddModal
          anime={adding}
          onClose={() => setAdding(null)}
          onAdded={handleRecommendationAdded}
        />
      )}
    </div>
  )
}

function AddModal({ anime, onClose, onAdded }) {
  const [status, setStatus] = useState("planned")
  const [rating, setRating] = useState("")
  const [favorite, setFavorite] = useState(false)
  const [success, setSuccess] = useState(false)
  const [error, setError] = useState(null)
  const [submitting, setSubmitting] = useState(false)

  async function handleAdd() {
    if (submitting) return

    setError(null)
    setSubmitting(true)

    try {
      const res = await api.get(`/anime/id/${anime.anilist_id}`)
      const animeData = res.data

      await api.post("/list/", {
        anime_id: animeData.id,
        status,
        rating: rating ? parseInt(rating) : null,
        is_favorite: favorite,
      })

      onAdded(anime.anilist_id)
      setSuccess(true)
    } catch (err) {
      const statusCode = err?.response?.status
      const detail = err?.response?.data?.detail

      console.error("Erro ao adicionar recomendação:", statusCode, err?.response?.data || err)

      if (statusCode === 400 && detail?.toLowerCase?.().includes("already")) {
        setError("esse anime já está na sua lista.")
        return
      }

      if (statusCode === 404) {
        setError("anime não encontrado no backend. tente abrir o card e adicionar de novo.")
        return
      }

      if (statusCode === 429) {
        setError("a AniList limitou as buscas agora. tente novamente em alguns minutos.")
        return
      }

      setError(detail || "erro ao adicionar. talvez já esteja na lista.")
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.8)",
      display: "flex", alignItems: "center", justifyContent: "center",
      zIndex: 50, padding: "1rem"
    }}>
      <div className="pixel-box" style={{ width: "100%", maxWidth: "400px" }}>

        <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem", flexWrap: "wrap" }}>
          {anime.cover_image_url && (
            <img
              src={anime.cover_image_url}
              alt={anime.title_romaji}
              style={{ width: "80px", border: "3px solid #e8d5b7", alignSelf: "flex-start" }}
            />
          )}
          <div style={{ flex: 1 }}>
            <h2 className="font-pixel" style={{ fontSize: "14px", color: "#a8c5a0", lineHeight: "1.6", marginBottom: "0.5rem" }}>
              {anime.title_english || anime.title_romaji}
            </h2>
            <p style={{ fontSize: "12px", color: "#c9a87c" }}>
              {anime.episodes ? `${anime.episodes} eps` : "eps: —"}
            </p>
            <p style={{ fontSize: "12px", color: "#a8a8c0" }}>
              {anime.genres?.join(" · ")}
            </p>
          </div>
        </div>

        {success ? (
          <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
            <p style={{ fontSize: "14px", color: "#a8c5a0" }}>✓ adicionado à lista!</p>
            <button className="pixel-btn" onClick={onClose}>✕ fechar</button>
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
            <div>
              <p style={{ fontSize: "13px", color: "#c9a87c", marginBottom: "0.5rem" }}>status:</p>
              <select className="pixel-input" value={status} onChange={(e) => setStatus(e.target.value)}>
                <option value="planned">planejado</option>
                <option value="watching">assistindo</option>
                <option value="completed">completado</option>
                <option value="dropped">dropado</option>
              </select>
            </div>

            <div>
              <p style={{ fontSize: "13px", color: "#c9a87c", marginBottom: "0.5rem" }}>nota (1-10):</p>
              <input
                className="pixel-input"
                type="number"
                placeholder="opcional"
                value={rating}
                onChange={(e) => setRating(e.target.value)}
                min={1}
                max={10}
              />
            </div>

            <label style={{ display: "flex", alignItems: "center", gap: "0.75rem", fontSize: "13px", cursor: "pointer" }}>
              <input type="checkbox" checked={favorite} onChange={(e) => setFavorite(e.target.checked)} />
              <span style={{ color: "#e8d5b7" }}>♥ favorito</span>
            </label>

            {error && <p style={{ fontSize: "13px", color: "#e07070" }}>✗ {error}</p>}

            <div style={{ display: "flex", gap: "0.75rem" }}>
              <button
                className="pixel-btn"
                style={{ flex: 1, opacity: submitting ? 0.6 : 1 }}
                onClick={handleAdd}
                disabled={submitting}
              >
                {submitting ? "⌛ adicionando..." : "+ ADICIONAR"}
              </button>
              <button
                className="pixel-btn pixel-btn-danger"
                onClick={onClose}
                disabled={submitting}
              >
                ✕
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}