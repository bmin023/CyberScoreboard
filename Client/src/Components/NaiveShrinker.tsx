import React, { useEffect, useRef, useState } from "react";

interface ShrinkerProps {
  name?: string;
  className?: string;
  placeholder?: string;
}

// I made this thing a couple projects ago, and add it to everything now.
// It's just a text input, but it shrinks to the exact size of the text.
// Really useful for stylish inline inputs and autocomplete.
const NaiveShrinker: React.FC<ShrinkerProps> = ({
  name,
  className,
  placeholder="",
}) => {
  let ref = useRef<HTMLParagraphElement>(null);
  let iref = useRef<HTMLInputElement>(null);

  // Under the hood, it copies the text to a hidden element, it can then check
  // the width of the hidden element to determine the width of the text
  useEffect(() => {
    if (ref.current && iref.current) {
      if (placeholder !== "" && placeholder.length > iref.current.value.length) {
        ref.current.innerText = placeholder;
      } else {
        ref.current.innerText = iref.current.value;
      }
      const newWidth = ref.current.clientWidth;
      iref.current.style.width = `${newWidth + 10}px`;
    }
  });
  const onEdit = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (ref.current && iref.current) {
      if (
        placeholder !== "" &&
        placeholder.length > iref.current.value.length
      ) {
        ref.current.innerText = placeholder;
      } else {
        ref.current.innerText = iref.current.value;
      }
      const newWidth = ref.current.clientWidth;
      iref.current.style.width = `${newWidth + 10}px`;
    }
  };

  return (
    <div>
      <p ref={ref} className={"invisible absolute " + className}></p>
      <input
        ref={iref}
        className={className}
        name={name ? name : "text"}
        placeholder={placeholder}
        onChange={onEdit}
      />
    </div>
  );
};

export default NaiveShrinker;
