export default function timeout(millis: number): Promise<void> {
  return new Promise((resolve, _reject) => {
    setTimeout(() => {
      resolve();
    }, millis);
  });
}
