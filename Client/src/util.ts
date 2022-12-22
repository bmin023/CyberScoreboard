export const formatDate = (date: Date) => {
  let hours = date.getHours() > 12 ? date.getHours() - 12 : date.getHours();
  return `${hours}:${date.getMinutes()}:${date.getSeconds()}`;
};