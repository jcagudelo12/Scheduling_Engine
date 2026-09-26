## Entorno

| | |
|---|---|
| Fecha | 2026-09-26 16:21 |
| CPU | AMD Ryzen 7 2700 Eight-Core Processor (16 hilos) |
| Kernel | 7.0.0-34-generic |
| Modo de frecuencia | schedutil |
| Docker | 29.1.3 |
| Imagen Rust | 499ca5bcdcaa (159MB) |
| Imagen Java | 7f8586c8bd2b (579MB) |
| Java | openjdk version "25.0.4.1" 2026-08-18 LTS |
| k6 | k6 v2.3.0 (commit/e088784614, go1.27.1, linux/amd64) |
| Núcleos | motor `4,6`, NATS `2,3`, k6 `8-15` |
| Repeticiones | 3, orden alternado; 30s por tasa, 5 s de pausa entre tasas |

## Arranque y reposo (mediana)

| | Rust | Java |
|---|---|---|
| Arranque hasta `/ready` (ms) | 558 | 3973 |
| Memoria en reposo con catálogo (MiB) | 5 | 150 |
| CPUs detectadas por el motor | 2 | 2 |

## Primeras solicitudes en frío (200/s, 20 s)

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 200 | 1.03 | 1.10 | 1.20 | 1.14–1.66 | 3.40 | 0.0 % | 0 | 17 % | 8 |
| java | 200 | 1.75 | 3.07 | 5.48 | 5.28–5.94 | 35 | 0.0 % | 0 | 59 % | 183 |

## 250 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 250 | 1.02 | 1.08 | 1.12 | 1.12–1.14 | 3.78 | 0.0 % | 0 | 19 % | 9 |
| java | 250 | 1.41 | 1.67 | 3.32 | 3.30–3.34 | 6.73 | 0.0 % | 0 | 39 % | 184 |

## 500 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 500 | 0.94 | 1.02 | 1.08 | 1.07–1.09 | 10 | 0.0 % | 0 | 36 % | 10 |
| java | 500 | 1.23 | 1.43 | 2.92 | 2.89–3.09 | 6.41 | 0.0 % | 0 | 50 % | 189 |

## 1000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 1000 | 0.92 | 1.50 | 1.67 | 1.65–1.69 | 8.96 | 0.0 % | 0 | 67 % | 14 |
| java | 1000 | 0.72 | 1.54 | 2.88 | 2.85–2.90 | 10 | 0.0 % | 0 | 74 % | 196 |

## 2000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 2000 | 0.63 | 1.38 | 1.73 | 1.72–1.73 | 14 | 0.0 % | 0 | 88 % | 21 |
| java | 2000 | 0.60 | 1.53 | 2.91 | 2.83–3.12 | 20 | 0.0 % | 0 | 98 % | 211 |

## 3000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 3000 | 0.56 | 0.85 | 1.38 | 1.30–2.10 | 27 | 0.0 % | 0 | 101 % | 29 |
| java | 3000 | 0.62 | 2.25 | 13 | 10–22 | 41 | 0.0 % | 0 | 135 % | 233 |

## 4000 solicitudes/s

| | RPS logrado | p50 (ms) | p95 (ms) | p99 (ms) | p99 mín–máx | máx (ms) | errores | descartadas | CPU prom. | memoria máx. (MiB) |
|---|---|---|---|---|---|---|---|---|---|---|
| rust | 4000 | 0.50 | 0.77 | 4.16 | 3.64–4.20 | 34 | 0.0 % | 0 | 115 % | 36 |
| java | 3768 | 479 | 723 | 745 | 428–803 | 1794 | 0.0 % | 4192 | 177 % | 346 |

Mediana de 3 repeticiones. CPU sobre 100 % por núcleo asignado.
