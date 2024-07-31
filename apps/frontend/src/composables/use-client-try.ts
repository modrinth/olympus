function defaultErrorHandler(error: any) {
  addNotification({
    group: "main",
    title: "An error occurred",
    text: error?.data?.description || error.message || error || "Unknown error",
    type: "error",
  });
}

type AsyncFunction<TArgs extends unknown[], TResult> = (...args: TArgs) => Promise<TResult>;

type ErrorHandlerFunction = (err: unknown) => void | Promise<void>;

type FinishCallbackFunction = () => Promise<void> | void;

export async function useClientTry<TArgs extends unknown[], TResult>(
  fn: AsyncFunction<TArgs, TResult>,
  onFail: ErrorHandlerFunction = defaultErrorHandler,
  onFinish?: FinishCallbackFunction,
) {
  return async function (...args: TArgs) {
    startLoading();
    try {
      return await fn(...args);
    } catch (err) {
      if (onFail) {
        await onFail(err);
      } else {
        console.error("[CLIENT TRY ERROR]", err);
      }
    } finally {
      if (onFinish) await onFinish();
      stopLoading();
    }
  };
}
