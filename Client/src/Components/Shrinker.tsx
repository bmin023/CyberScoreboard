import React, { useEffect, useRef, useState } from "react";

interface ShrinkerProps {
  value: string;
  name?: string;
  onChange: (value: string) => void;
  className?: string;
}

// I made this thing a couple projects ago, and add it to everything now.
// It's just a text input, but it shrinks to the exact size of the text.
// Really useful for stylish inline inputs and autocomplete.
export const Shrinker: React.FC<ShrinkerProps> = ({ value, name, onChange, className }) => {
  let ref = useRef<HTMLParagraphElement>(null);
  let iref = useRef<HTMLInputElement>(null);

  // Under the hood, it copies the text to a hidden element, it can then check
  // the width of the hidden element to determine the width of the text
  useEffect(() => {
    if (ref.current && iref.current) {
      ref.current.innerText = iref.current.value;
      const newWidth = ref.current.clientWidth;
      iref.current.style.width = `${newWidth + 10}px`;
    }
  });
  const onEdit = (e : React.ChangeEvent<HTMLInputElement>) => {
    if (ref.current) {
      ref.current.innerText = e.target.value;
      const newWidth = ref.current.clientWidth;
      e.target.style.width = `${newWidth}px`;
    }
    onChange(e.target.value);
  }

  return (
    <div>
      <p ref={ref} className={ "invisible absolute " + className }></p>
      <input ref={iref} className={className} name={name ? name : "text"} onChange={onEdit} value={value}/>
    </div>
  );
};
