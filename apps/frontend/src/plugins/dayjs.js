import dayjs from "dayjs";
import quarterOfYear from "dayjs/plugin/quarterOfYear";

dayjs.extend(quarterOfYear);

export default defineNuxtPlugin(() => {
  return {
    provide: {
      dayjs,
    },
  };
});
