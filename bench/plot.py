#!/usr/bin/env python3
"""Gráficas de la comparación Rust vs Java, a partir de los resultados de bench/run.sh
y bench/micro.sh. Genera cada figura en versión clara y oscura (<nombre>.png y
<nombre>-dark.png) en la carpeta de resultados.

Uso: bench/.venv/bin/python bench/plot.py <carpeta> <repeticiones> <tasa> [<tasa> ...]
"""

import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import FancyBboxPatch, Rectangle  # noqa: E402
from matplotlib.ticker import FixedLocator, FuncFormatter, NullLocator  # noqa: E402

from results_lib import ENGINES, Results  # noqa: E402

# Paleta validada con el validador de daltonismo (ΔE ≥ 24 entre las dos series en ambos modos).
THEMES = {
    "light": {
        "surface": "#fcfcfb", "ink": "#0b0b0b", "ink2": "#52514e", "muted": "#898781",
        "grid": "#e1e0d9", "axis": "#c3c2b7", "rust": "#2a78d6", "java": "#eb6834",
    },
    "dark": {
        "surface": "#1a1a19", "ink": "#ffffff", "ink2": "#c3c2b7", "muted": "#898781",
        "grid": "#2c2c2a", "axis": "#383835", "rust": "#3987e5", "java": "#d95926",
    },
}
NAMES = {"rust": "Rust", "java": "Java"}
DPI = 192  # exporta al doble de resolución para pantallas de alta densidad


def px(n):
    """Tamaño en píxeles de pantalla (CSS) a puntos de matplotlib."""
    return n * 0.75


def es_num(value, decimals=0):
    """Número con separador de miles y coma decimal, como se escribe en español."""
    text = f"{value:,.{decimals}f}"
    return text.replace(",", "X").replace(".", ",").replace("X", ".")


def style(fig, axes, t):
    fig.patch.set_facecolor(t["surface"])
    for ax in axes:
        ax.set_facecolor(t["surface"])
        for side in ("top", "right"):
            ax.spines[side].set_visible(False)
        for side in ("left", "bottom"):
            ax.spines[side].set_color(t["axis"])
            ax.spines[side].set_linewidth(px(1))
        ax.tick_params(colors=t["muted"], labelcolor=t["ink2"], labelsize=9, length=0, pad=6)
        ax.grid(True, color=t["grid"], linewidth=px(1))
        ax.set_axisbelow(True)


def titles(fig, t, title, subtitle):
    """Título y subtítulo a distancias fijas en píxeles del borde superior."""
    height_px = fig.get_figheight() * 96
    fig.text(0.02, 1 - 14 / height_px, title, ha="left", va="top", fontsize=14, fontweight="bold", color=t["ink"])
    fig.text(0.02, 1 - 42 / height_px, subtitle, ha="left", va="top", fontsize=9, color=t["ink2"])


def save(fig, out, name, mode):
    path = out / (f"{name}.png" if mode == "light" else f"{name}-dark.png")
    fig.savefig(path, dpi=DPI, facecolor=fig.get_facecolor())
    plt.close(fig)
    return path


def rate_axis(ax, rates, t):
    ax.set_xlim(rates[0] * 0.8, rates[-1] * 1.22)
    ax.xaxis.set_major_locator(FixedLocator(rates))
    ax.xaxis.set_major_formatter(FuncFormatter(lambda v, _: es_num(v)))
    ax.set_xlabel("Solicitudes por segundo", color=t["ink2"], fontsize=9, labelpad=8)
    ax.grid(False, axis="x")


def line_series(ax, rates, values, color, t):
    ax.plot(rates, values, color=color, linewidth=px(2), zorder=3)
    # Marcadores con anillo del color de la superficie para separarlos de la línea vecina.
    ax.plot(rates, values, "o", color=color, markersize=px(9), markeredgecolor=t["surface"],
            markeredgewidth=px(2), zorder=4)


def end_label(ax, x, y, text, t, dy=0):
    ax.annotate(text, (x, y), xytext=(10, dy), textcoords="offset points", va="center",
                ha="left", fontsize=9, color=t["ink"], fontweight="bold")


def legend(ax, t, loc="upper left"):
    leg = ax.legend(loc=loc, frameon=False, fontsize=9, labelcolor=t["ink2"], handlelength=1.6)
    return leg


def latency_chart(results, rates, out, mode):
    t = THEMES[mode]
    fig, ax = plt.subplots(figsize=(880 / 96, 440 / 96))
    fig.subplots_adjust(left=0.09, right=0.86, top=0.8, bottom=0.14)
    style(fig, [ax], t)

    ends = {}
    for engine in ENGINES:
        phases = [results.phase(engine, f"r{r}") for r in rates]
        p99 = [p["p99"] for p in phases]
        lo = [p["p99_min"] for p in phases]
        hi = [p["p99_max"] for p in phases]
        color = t[engine]
        ax.fill_between(rates, lo, hi, color=color, alpha=0.18, linewidth=0, zorder=2)
        line_series(ax, rates, p99, color, t)
        ax.plot([], [], color=color, linewidth=px(2), marker="o", markersize=px(9), label=NAMES[engine])
        ends[engine] = p99[-1]

    for engine, value in ends.items():
        text = f"{NAMES[engine]}  {es_num(value, 1) if value < 10 else es_num(value)} ms"
        end_label(ax, rates[-1], value, text, t)

    ax.set_yscale("log")
    # El eje llega a la siguiente potencia de 10 por encima del valor más alto.
    top = max(max(p["p99_max"] for e in ENGINES for p in (results.phase(e, f"r{r}") for r in rates)), 5)
    decades = [d for d in (1, 10, 100, 1000) if d <= top * 10]
    ax.set_ylim(min(0.2, top), 10 ** (len(str(int(top)))) * 2)
    ax.yaxis.set_major_locator(FixedLocator(decades))
    ax.yaxis.set_minor_locator(NullLocator())
    ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: "1 s" if v >= 1000 else f"{v:g} ms"))
    rate_axis(ax, rates, t)

    java_last = results.phase("java", f"r{rates[-1]}")
    if java_last and java_last["dropped"] > 0:
        ax.annotate(f"Java se satura: {es_num(java_last['dropped'])} solicitudes\nsin atender a {es_num(rates[-1])}/s",
                    (rates[-1], java_last["p99"]), xytext=(-12, -2), textcoords="offset points",
                    ha="right", va="top", fontsize=8.5, color=t["ink2"])

    legend(ax, t)
    titles(fig, t, "Latencia p99 según la carga (menos es mejor)",
           f"Mediana de {len(results.reps)} repeticiones · 2 CPUs y 1 GB por motor · "
           "banda: rango entre repeticiones · escala logarítmica")
    return save(fig, out, "latency-p99", mode)


def cpu_chart(results, rates, out, mode):
    t = THEMES[mode]
    fig, ax = plt.subplots(figsize=(880 / 96, 400 / 96))
    fig.subplots_adjust(left=0.09, right=0.86, top=0.79, bottom=0.15)
    style(fig, [ax], t)

    ax.axhline(200, color=t["axis"], linewidth=px(1.5), linestyle=(0, (4, 3)), zorder=2)
    ax.annotate("límite: 2 núcleos", (rates[-1] * 1.2, 200), xytext=(0, 5), textcoords="offset points",
                fontsize=8.5, color=t["muted"], va="bottom", ha="right")

    for engine in ENGINES:
        cpu = [results.phase(engine, f"r{r}")["cpu"] for r in rates]
        line_series(ax, rates, cpu, t[engine], t)
        ax.plot([], [], color=t[engine], linewidth=px(2), marker="o", markersize=px(9), label=NAMES[engine])
        end_label(ax, rates[-1], cpu[-1], f"{NAMES[engine]}  {cpu[-1]:.0f} %", t)

    ax.set_ylim(0, 230)
    ax.yaxis.set_major_locator(FixedLocator([0, 50, 100, 150, 200]))
    ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:.0f} %"))
    rate_axis(ax, rates, t)
    legend(ax, t, loc="lower right")
    titles(fig, t, "Uso de CPU según la carga (menos es mejor)",
           "Promedio durante cada tasa · 100 % = un núcleo completo")
    return save(fig, out, "cpu", mode)


def hbars(ax, values, t, unit, decimals=0):
    """Barras horizontales, una por motor, con el extremo de datos redondeado."""
    fig = ax.figure
    ax.set_ylim(-0.6, len(ENGINES) - 0.4)
    ax.set_xlim(0, max(values.values()) * 1.35)
    ax.invert_yaxis()
    fig.canvas.draw()
    # Radio de 4 px convertido a unidades de datos en cada eje.
    (x0, y0), (x1, y1) = ax.transData.inverted().transform([(0, 0), (px(4) * DPI / 72, px(4) * DPI / 72)])
    rx, ry = abs(x1 - x0), abs(y1 - y0)
    height = 0.62
    for i, engine in enumerate(ENGINES):
        value = values[engine]
        top = i - height / 2
        radius = min(rx, value / 2)
        ax.add_patch(FancyBboxPatch((0, top), value, height, boxstyle=f"round,pad=0,rounding_size={radius}",
                                    mutation_aspect=ry / rx, facecolor=t[engine], edgecolor="none",
                                    antialiased=True, zorder=3))
        # La mitad del lado del eje se cubre con un rectángulo: solo el extremo del dato queda redondeado.
        ax.add_patch(Rectangle((0, top), value / 2, height, facecolor=t[engine], edgecolor="none", zorder=3))
        label = f"{es_num(value, decimals)} {unit}"
        ax.annotate(label, (value, i), xytext=(6, 0), textcoords="offset points", va="center",
                    fontsize=10, fontweight="bold", color=t["ink"])
    ax.set_yticks(range(len(ENGINES)), [NAMES[e] for e in ENGINES])
    ax.tick_params(axis="y", labelsize=10, labelcolor=t["ink"])
    ax.grid(False, axis="y")
    ax.set_xticks([])
    ax.spines["bottom"].set_visible(False)


def panel_title(ax, text, t):
    ax.set_title(text, loc="left", fontsize=10, color=t["ink2"], pad=10)


def resources_chart(results, rates, out, mode):
    t = THEMES[mode]
    fig, axes = plt.subplots(1, 3, figsize=(880 / 96, 300 / 96))
    fig.subplots_adjust(left=0.07, right=0.97, top=0.66, bottom=0.08, wspace=0.45)
    style(fig, axes, t)

    startup = {e: results.startup_ms(e) / 1000 for e in ENGINES}
    idle = {e: results.idle_mem_mib(e) for e in ENGINES}
    peak = {e: max(results.phase(e, f"r{r}")["mem"] for r in rates) for e in ENGINES}

    panel_title(axes[0], "Arranque hasta estar listo", t)
    hbars(axes[0], startup, t, "s", decimals=2)
    panel_title(axes[1], "Memoria en reposo", t)
    hbars(axes[1], idle, t, "MB")
    panel_title(axes[2], "Memoria máxima bajo carga", t)
    hbars(axes[2], peak, t, "MB")

    titles(fig, t, "Arranque y memoria (menos es mejor)",
           f"Mediana de {len(results.reps)} repeticiones · memoria con el catálogo de 5.000 grupos cargado")
    return save(fig, out, "resources", mode)


def micro_chart(results, out, mode):
    micro = results.micro()
    if micro is None:
        return None
    t = THEMES[mode]
    fig, axes = plt.subplots(1, 2, figsize=(880 / 96, 280 / 96))
    fig.subplots_adjust(left=0.07, right=0.96, top=0.62, bottom=0.08, wspace=0.35)
    style(fig, axes, t)

    panel_title(axes[0], "Backtracking: 6 cursos × 5 grupos", t)
    hbars(axes[0], micro["backtracking_6x5"], t, "µs")
    panel_title(axes[1], "Buscar 6 cursos en un catálogo de 5.000 grupos", t)
    hbars(axes[1], micro["catalog_lookup_5000"], t, "µs", decimals=2)

    titles(fig, t, "Solo el cálculo, sin HTTP (menos es mejor)",
           "Microbenchmarks: criterion en Rust, JMH en Java · mediana")
    return save(fig, out, "micro", mode)


def main():
    out = Path(sys.argv[1])
    results = Results(out, sys.argv[2])
    rates = [int(r) for r in sys.argv[3:]]
    for mode in THEMES:
        for path in (
            latency_chart(results, rates, out, mode),
            cpu_chart(results, rates, out, mode),
            resources_chart(results, rates, out, mode),
            micro_chart(results, out, mode),
        ):
            if path:
                print(path)


if __name__ == "__main__":
    main()
