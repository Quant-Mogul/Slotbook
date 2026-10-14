"use client";

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { gsap } from "gsap";

type Theme = "dark" | "light";
const CELESTIAL_TRANSITION_MS = 820;

function getInitialTheme(): Theme {
  if (typeof window === "undefined") return "dark";
  const saved = window.localStorage.getItem("slotbook-theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>("dark");
  const previewTheme: Theme = theme;
  const earthRef = useRef<HTMLSpanElement>(null);
  const illuminationRef = useRef<HTMLSpanElement>(null);
  const orbitRef = useRef<HTMLSpanElement>(null);
  const sunRef = useRef<HTMLSpanElement>(null);
  const transitionTimerRef = useRef<number | null>(null);
  const isTransitioningRef = useRef(false);

  useEffect(() => {
    const initialThemeTimer = window.setTimeout(() => {
      const initialTheme = getInitialTheme();
      if (!isTransitioningRef.current) document.documentElement.dataset.theme = initialTheme;
      if (initialTheme !== "dark") setTheme(initialTheme);
    }, 0);
    const mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
    if (window.localStorage.getItem("slotbook-theme")) {
      return () => window.clearTimeout(initialThemeTimer);
    }

    const handleSystemThemeChange = (event: MediaQueryListEvent) => {
      const nextTheme: Theme = event.matches ? "light" : "dark";
      document.documentElement.dataset.theme = nextTheme;
      setTheme(nextTheme);
    };

    mediaQuery.addEventListener("change", handleSystemThemeChange);
    return () => {
      window.clearTimeout(initialThemeTimer);
      mediaQuery.removeEventListener("change", handleSystemThemeChange);
    };
  }, []);

  useEffect(() => {
    if (!isTransitioningRef.current) document.documentElement.dataset.theme = theme;
  }, [theme]);

  useLayoutEffect(() => {
    const earth = earthRef.current;
    const illumination = illuminationRef.current;
    const orbit = orbitRef.current;
    const sun = sunRef.current;
    if (
      !earth ||
      !illumination ||
      !orbit ||
      !sun ||
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    )
      return;

    const isLightPreview = previewTheme === "light";
    const startAngle = isLightPreview ? 232 : 52;
    const sideAngle = 125;
    const endAngle = isLightPreview ? 52 : 232;
    gsap.killTweensOf([earth, illumination, orbit, sun]);
    gsap.set(orbit, { rotationY: startAngle });
    gsap.set(sun, {
      autoAlpha: isLightPreview ? 0 : 1,
      scale: isLightPreview ? 0.55 : 1.15,
      z: isLightPreview ? -18 : 18,
      rotationX: 20,
      rotationY: -startAngle,
      rotationZ: 12,
    });

    const transition = gsap.timeline();
    transition.to(orbit, {
      rotationY: sideAngle,
      duration: 0.24,
      ease: "power1.inOut",
      force3D: true,
    });
    transition.to(
      sun,
      {
        autoAlpha: 0.72,
        scale: 0.82,
        z: 0,
        rotationY: -sideAngle,
        duration: 0.24,
        ease: "power1.inOut",
        force3D: true,
      },
      "<",
    );
    transition.to(orbit, {
      rotationY: endAngle,
      duration: 0.56,
      ease: "power3.inOut",
      force3D: true,
    });
    transition.to(
      sun,
      {
        autoAlpha: isLightPreview ? 1 : 0,
        scale: isLightPreview ? 1.3 : 0.45,
        z: isLightPreview ? 24 : -24,
        rotationY: -endAngle,
        duration: 0.56,
        ease: "power3.inOut",
        force3D: true,
      },
      "<",
    );
    transition.to(
      earth,
      { scale: isLightPreview ? 1.08 : 0.94, duration: 0.58, ease: "power2.out", force3D: true },
      0.16,
    );
    transition.to(
      illumination,
      {
        clipPath: isLightPreview
          ? "ellipse(100% 100% at 100% 50%)"
          : "ellipse(0% 100% at 100% 50%)",
        duration: 0.68,
        ease: "power2.inOut",
      },
      0.16,
    );

    return () => {
      transition.kill();
      gsap.killTweensOf([earth, illumination, orbit, sun]);
    };
  }, [previewTheme]);

  function toggleTheme() {
    const nextTheme: Theme = theme === "dark" ? "light" : "dark";
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      document.documentElement.dataset.theme = nextTheme;
      window.localStorage.setItem("slotbook-theme", nextTheme);
      setTheme(nextTheme);
      return;
    }
    isTransitioningRef.current = true;
    if (transitionTimerRef.current) window.clearTimeout(transitionTimerRef.current);
    setTheme(nextTheme);
    transitionTimerRef.current = window.setTimeout(() => {
      document.documentElement.dataset.theme = nextTheme;
      window.localStorage.setItem("slotbook-theme", nextTheme);
      isTransitioningRef.current = false;
    }, CELESTIAL_TRANSITION_MS);
  }

  const nextLabel = theme === "dark" ? "Switch to light mode" : "Switch to dark mode";

  return (
    <button
      className="theme-toggle"
      type="button"
      onClick={toggleTheme}
      aria-label={nextLabel}
      title={nextLabel}
      suppressHydrationWarning
    >
      <span className="celestial-scene" aria-hidden="true">
        <span ref={earthRef} className={`celestial-earth ${previewTheme}`}>
          <span ref={illuminationRef} className="earth-illumination" />
        </span>
        <span ref={orbitRef} className="celestial-orbit">
          <span ref={sunRef} className="celestial-sun" />
        </span>
      </span>
    </button>
  );
}
