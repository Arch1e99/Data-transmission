#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Rysowanie wykresow 3D BER(alpha, beta) z plikow CSV wygenerowanych przez czesc
obliczeniowa w Rust. Styl wzorowany na wykresach referencyjnych (matplotlib).

Uzycie:
    python plot.py <katalog_z_csv> <katalog_wyjsciowy>
np.
    python plot.py lab-13/data        lab-13
    python plot.py lab-13/data_fixed  lab-13/fixed
"""

import sys
import os
import glob
import numpy as np
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D  # noqa: F401


def read_csv(path):
    with open(path, "r", encoding="utf-8") as f:
        lines = [ln.strip() for ln in f if ln.strip()]
    alpha_max = float(lines[0].split(",")[1])
    beta_max = float(lines[1].split(",")[1])
    grid = np.array([[float(v) for v in ln.split(",")] for ln in lines[2:]])
    # grid[ia][ib] -> wiersze=alpha, kolumny=beta
    return alpha_max, beta_max, grid


def nice_title(basename):
    # ber_ASK_Hamming74_I+II -> "ASK, Hamming(7,4) | I+II (szum/tlumienie)"
    name = basename[len("ber_"):] if basename.startswith("ber_") else basename
    parts = name.split("_")
    mod = parts[0]
    code = parts[1]
    cfg = parts[2] if len(parts) > 2 else ""
    code_map = {"Hamming74": "Hamming(7,4)", "Hamming1511": "Hamming(15,11)"}
    code_str = code_map.get(code, code)
    cfg_desc = {
        "I+II": "I+II (szum->tlumienie)",
        "II+I": "II+I (tlumienie->szum)",
    }.get(cfg, cfg)
    return f"{mod}, {code_str} | {cfg_desc}"


def plot_one(path, out_dir):
    base = os.path.splitext(os.path.basename(path))[0]
    alpha_max, beta_max, grid = read_csv(path)
    na, nb = grid.shape

    alpha = np.linspace(0.0, alpha_max, na)
    beta = np.linspace(0.0, beta_max, nb)
    # meshgrid: X=alpha (os pozioma), Y=beta (glebia)
    A, B = np.meshgrid(alpha, beta, indexing="ij")  # zgodne z grid[ia][ib]
    Z = grid

    fig = plt.figure(figsize=(8, 6))
    ax = fig.add_subplot(111, projection="3d")

    # BER jest nieujemny -> normalizacja koloru od 0; unikamy ujemnych etykiet.
    zmax = float(Z.max())
    vmax = zmax if zmax > 1e-9 else 1e-3

    surf = ax.plot_surface(
        A,
        B,
        Z,
        cmap="plasma",
        vmin=0.0,
        vmax=vmax,
        edgecolor="none",
        rstride=1,
        cstride=1,
        antialiased=True,
        linewidth=0,
    )

    ax.set_xlabel("alfa (szum)", labelpad=10)
    ax.set_ylabel("beta (tlumienie)", labelpad=10)
    ax.set_zlabel("BER", labelpad=6)
    ax.set_title(nice_title(base), pad=15)

    # Zakres osi BER zawsze od 0 (z lekkim marginesem u gory).
    if zmax < 1e-9:
        ax.set_zlim(0.0, 1e-3)
    else:
        ax.set_zlim(0.0, zmax * 1.1)

    ax.view_init(elev=25, azim=-60)
    cb = fig.colorbar(surf, ax=ax, shrink=0.6, aspect=14, pad=0.1)
    cb.set_label("BER")

    fig.tight_layout()
    out_path = os.path.join(out_dir, base + ".png")
    fig.savefig(out_path, dpi=130)
    plt.close(fig)
    return out_path


def main():
    if len(sys.argv) < 3:
        print("Uzycie: python plot.py <katalog_csv> <katalog_wyjsciowy>")
        sys.exit(1)
    data_dir = sys.argv[1]
    out_dir = sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)

    files = sorted(glob.glob(os.path.join(data_dir, "*.csv")))
    if not files:
        print(f"Brak plikow CSV w {data_dir}")
        sys.exit(1)

    for i, path in enumerate(files, 1):
        out = plot_one(path, out_dir)
        print(f"[{i:2}/{len(files)}] {out}")

    print(f"Gotowe. {len(files)} wykresow w {out_dir}/")


if __name__ == "__main__":
    main()
