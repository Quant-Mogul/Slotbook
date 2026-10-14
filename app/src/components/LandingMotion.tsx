"use client";

import { useLayoutEffect } from "react";
import { createGsapMotion } from "@/lib/gsap-motion";

export function LandingMotion() {
  useLayoutEffect(
    () =>
      createGsapMotion((motion) => {
        motion.from(".hero-copy > *", {
          y: 28,
          autoAlpha: 0,
          duration: 0.8,
          stagger: 0.09,
          ease: "power3.out",
          delay: 0.1,
        });
        motion.to(".metric-grid, .flow-section .stage-grid", {
          yPercent: -5,
          ease: "none",
          scrollTrigger: {
            trigger: ".flow-section",
            start: "top bottom",
            end: "bottom top",
            scrub: 1.5,
          },
        });
        motion.utils
          .toArray<HTMLElement>(".section-intro, .metric, .stage, .data-card, .data-copy, .closing")
          .forEach((element) => {
            motion.from(element, {
              y: 30,
              autoAlpha: 0,
              duration: 0.7,
              ease: "power2.out",
              scrollTrigger: { trigger: element, start: "top 86%", once: true },
            });
          });
        motion.from(".data-card .card-top, .data-card .main-row", {
          y: 18,
          autoAlpha: 0,
          duration: 0.65,
          stagger: 0.14,
          ease: "power3.out",
          scrollTrigger: { trigger: ".data-card", start: "top 82%", once: true },
        });
        motion.from(".data-card .data-columns > div", {
          y: 22,
          autoAlpha: 0,
          duration: 0.55,
          stagger: 0.1,
          ease: "power2.out",
          scrollTrigger: { trigger: ".data-columns", start: "top 86%", once: true },
        });
        motion.from(".data-card .progress", {
          scaleX: 0,
          transformOrigin: "left center",
          duration: 0.9,
          ease: "power3.out",
          scrollTrigger: { trigger: ".progress", start: "top 88%", once: true },
        });
        motion.from(".data-card .card-footer", {
          y: 14,
          autoAlpha: 0,
          duration: 0.6,
          ease: "power2.out",
          scrollTrigger: { trigger: ".card-footer", start: "top 90%", once: true },
        });
        motion.from(".closing h2, .closing .angular-button", {
          y: 24,
          autoAlpha: 0,
          duration: 0.8,
          stagger: 0.12,
          ease: "power3.out",
          scrollTrigger: { trigger: ".closing", start: "top 78%", once: true },
        });
        motion.from(".closing .eyebrow", {
          y: 18,
          autoAlpha: 0,
          duration: 0.6,
          ease: "power2.out",
          scrollTrigger: { trigger: ".closing", start: "top 82%", once: true },
        });
      }),
    [],
  );

  return null;
}
