#!/usr/bin/env bash
# Compara la versión en Rust y la versión en Java en condiciones idénticas.
#
#   bench/run.sh                                  # 3 repeticiones, tasas por defecto
#   REPS=5 RATES="500 1000" DURATION=60s bench/run.sh
#
# Cada corrida (motor × repetición) parte de cero: NATS vacío, un solo motor encendido, los
# mismos núcleos, el mismo catálogo y los mismos estudiantes en el mismo orden. El orden de
# los motores se alterna entre repeticiones. Resultado: bench/results/report.md.
set -euo pipefail
cd "$(dirname "$0")/.."

REPS=${REPS:-3}
RATES=${RATES:-"250 500 1000 2000 3000 4000"}
DURATION=${DURATION:-30s}
COOLDOWN=${COOLDOWN:-5}
export ENGINE_CPUS=${ENGINE_CPUS:-4,6}   # 2 núcleos físicos; sus hermanos (5, 7) quedan libres
export NATS_CPUS=${NATS_CPUS:-2,3}
K6_CPUS=${K6_CPUS:-8-15}
EXPECTED_CPUS=$(( $(tr ',' '\n' <<< "$ENGINE_CPUS" | wc -l) ))

C="docker compose -f bench/docker-compose.yml"
OUT=bench/results
declare -A HTTP=([rust]=18080 [java]=28080)
declare -A GRPC=([rust]=15051 [java]=25051)

mkdir -p "$OUT"
# Se conservan los microbenchmarks (micro.json, jmh.json), que genera bench/micro.sh.
find "$OUT" -maxdepth 1 -type f \( -name '*_rep*' -o -name 'environment.md' -o -name 'report.md' \) -delete

echo "==> Datos con semilla fija"
python3 bench/generate_scenario.py
cargo build -q --manifest-path rust/Cargo.toml -p sched-simulation --bin fake-institution

wait_http() { # url código [método] [cuerpo] — espera hasta 120 s
  local deadline=$(( $(date +%s) + 120 ))
  until [ "$(curl -s -o /dev/null -w '%{http_code}' -X "${3:-GET}" "$1" -H 'content-type: application/json' ${4:+-d "$4"})" = "$2" ]; do
    if [ "$(date +%s)" -ge "$deadline" ]; then echo "timeout esperando $1" >&2; exit 1; fi
    sleep 0.05
  done
}

# Muestra CPU y memoria del contenedor mientras corre k6.
sample() { # contenedor archivo
  while :; do docker stats --no-stream --format '{{.CPUPerc}} {{.MemUsage}}' "$1" >> "$2"; done
}

run_k6() { # motor prefijo tasa duración nombre
  local engine=$1 prefix=$2 rate=$3 duration=$4 name=$5
  sample "bench-$engine-1" "$OUT/${prefix}_${name}_stats.txt" & local sampler=$!
  docker run --rm --network bench_default --cpuset-cpus "$K6_CPUS" -u "$(id -u):$(id -g)" \
    -v "$PWD/bench:/bench" -w /bench \
    -e TARGET="http://$engine:8080" -e RATE="$rate" -e DURATION="$duration" \
    -e OUT="/bench/results/${prefix}_${name}.json" \
    grafana/k6 run --quiet load.js >/dev/null 2>&1 || true
  kill "$sampler" 2>/dev/null; wait "$sampler" 2>/dev/null || true
}

record_environment() {
  {
    echo "## Entorno"
    echo
    echo "| | |"
    echo "|---|---|"
    echo "| Fecha | $(date '+%Y-%m-%d %H:%M') |"
    echo "| CPU | $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs) ($(nproc) hilos) |"
    echo "| Kernel | $(uname -r) |"
    echo "| Modo de frecuencia | $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo desconocido) |"
    echo "| Docker | $(docker version --format '{{.Server.Version}}') |"
    echo "| Imagen Rust | $(docker image inspect -f '{{.Id}}' sched-engine:dev | cut -c8-19) ($(docker image ls -f reference=sched-engine:dev --format '{{.Size}}')) |"
    echo "| Imagen Java | $(docker image inspect -f '{{.Id}}' sched-engine-java:dev | cut -c8-19) ($(docker image ls -f reference=sched-engine-java:dev --format '{{.Size}}')) |"
    echo "| Java | $(docker run --rm --entrypoint java sched-engine-java:dev -version 2>&1 | head -1) |"
    echo "| k6 | $(docker run --rm grafana/k6 version 2>/dev/null | head -1) |"
    echo "| Núcleos | motor \`$ENGINE_CPUS\`, NATS \`$NATS_CPUS\`, k6 \`$K6_CPUS\` |"
    echo "| Repeticiones | $REPS, orden alternado; $DURATION por tasa, $COOLDOWN s de pausa entre tasas |"
  } > "$OUT/environment.md"
}

record_environment

for rep in $(seq 1 "$REPS"); do
  if (( rep % 2 == 1 )); then order="rust java"; else order="java rust"; fi
  for engine in $order; do
    prefix="${engine}_rep${rep}"
    echo "==> Repetición $rep/$REPS: $engine"

    # Estado inicial idéntico: todo apagado, NATS vacío, solo este motor.
    $C down -v >/dev/null 2>&1 || true
    $C up -d nats >/dev/null 2>&1
    sleep 1
    cut -d' ' -f1-3 /proc/loadavg > "$OUT/${prefix}_loadavg.txt"

    start=$(date +%s%N)
    $C up -d "$engine" >/dev/null 2>&1
    wait_http "http://localhost:${HTTP[$engine]}/ready" 200
    echo $(( ($(date +%s%N) - start) / 1000000 )) > "$OUT/${prefix}_startup_ms.txt"

    cpus=$($C logs "$engine" 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | grep -oE 'cpus[=:] ?[0-9]+' | grep -oE '[0-9]+$' | head -1 || true)
    echo "${cpus:-?}" > "$OUT/${prefix}_cpus.txt"
    if [ "${cpus:-0}" != "$EXPECTED_CPUS" ]; then
      echo "    AVISO: $engine detectó ${cpus:-?} CPUs; se esperaban $EXPECTED_CPUS" >&2
    fi

    # El gRPC de ingesta abre justo después de /ready: se reintenta como lo haría el SDK.
    for attempt in $(seq 1 50); do
      SCHED_INGEST_URL="http://localhost:${GRPC[$engine]}" \
        rust/target/debug/fake-institution load bench/scenario.json >/dev/null 2>&1 && break
      [ "$attempt" = 50 ] && { echo "no se pudo cargar el catálogo en $engine" >&2; exit 1; }
      sleep 0.2
    done
    wait_http "http://localhost:${HTTP[$engine]}/combinations" 200 POST \
      '{"student_id":"x","eligible_courses":["C0001"]}'
    docker stats --no-stream --format '{{.MemUsage}}' "bench-$engine-1" | cut -d/ -f1 > "$OUT/${prefix}_idle_mem.txt"

    echo "    primeras solicitudes en frío (200/s, 20 s)"
    run_k6 "$engine" "$prefix" 200 20s cold
    for rate in $RATES; do
      sleep "$COOLDOWN"
      echo "    $rate solicitudes/s durante $DURATION"
      run_k6 "$engine" "$prefix" "$rate" "$DURATION" "r$rate"
    done
  done
done

$C down -v >/dev/null 2>&1
python3 bench/report.py "$OUT" "$REPS" $RATES | tee "$OUT/report.md"

# Gráficas (requieren el entorno de bench/requirements.txt).
if [ -x bench/.venv/bin/python ]; then
  bench/.venv/bin/python bench/plot.py "$OUT" "$REPS" $RATES >/dev/null && echo "==> Gráficas en $OUT/*.png"
else
  echo "==> Para generar gráficas: python3 -m venv bench/.venv && bench/.venv/bin/pip install -r bench/requirements.txt"
fi
