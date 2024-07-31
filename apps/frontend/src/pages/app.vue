<script setup>
import {
  TrashIcon,
  SearchIcon,
  BoxIcon,
  SendIcon,
  EditIcon,
  DownloadIcon,
  LinkIcon,
} from "@modrinth/assets";
import { Avatar, AnimatedLogo, Badge, Checkbox } from "@modrinth/ui";
import PrismIcon from "~/assets/images/external/prism.svg?component";
import ATLauncher from "~/assets/images/external/atlauncher.svg?component";
import CurseForge from "~/assets/images/external/curseforge.svg?component";

import { homePageProjects } from "~/generated/state.json";

const os = ref(null);
const macValue = ref(null);
const downloadWindows = ref(null);
const downloadLinux = ref(null);
const downloadSection = ref(null);
const windowsLink = ref(null);
const linuxLinks = {
  appImage: null,
  deb: null,
  thirdParty: "https://support.modrinth.com/en/articles/9298760",
};
const macLinks = {
  appleSilicon: null,
  intel: null,
};

let downloadLauncher;

const newProjects = homePageProjects.slice(0, 40);
const val = Math.ceil(newProjects.length / 6);
const rows = ref([
  newProjects.slice(0, val),
  newProjects.slice(val, val * 2),
  newProjects.slice(val * 2, val * 3),
  newProjects.slice(val * 3, val * 4),
  newProjects.slice(val * 4, val * 5),
]);

const [{ data: launcherUpdates }] = await Promise.all([
  await useAsyncData("launcherUpdates", () =>
    $fetch("https://launcher-files.modrinth.com/updates.json"),
  ),
]);

macLinks.appleSilicon = launcherUpdates.value.platforms["darwin-aarch64"].install_urls[0];
macLinks.intel = launcherUpdates.value.platforms["darwin-x86_64"].install_urls[0];
windowsLink.value = launcherUpdates.value.platforms["windows-x86_64"].install_urls[0];
linuxLinks.appImage = launcherUpdates.value.platforms["linux-x86_64"].install_urls[1];
linuxLinks.deb = launcherUpdates.value.platforms["linux-x86_64"].install_urls[0];

onMounted(() => {
  os.value = navigator?.platform.toString();
  os.value = os.value?.includes("Mac")
    ? "Mac"
    : os.value?.includes("Win")
      ? "Windows"
      : os.value?.includes("Linux")
        ? "Linux"
        : null;

  if (os.value === "Windows") {
    downloadLauncher = () => {
      downloadWindows.value.click();
    };
  } else if (os.value === "Linux") {
    downloadLauncher = () => {
      downloadLinux.value.click();
    };
  } else {
    downloadLauncher = () => {
      scrollToSection();
    };
  }
});

watch(macValue, () => {
  if (macValue.value === "Download for Apple Silicon") {
    const link = document.createElement("a");
    link.href = macLinks.appleSilicon;
    link.download = "";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  } else if (macValue.value === "Download for Intel") {
    const link = document.createElement("a");
    link.href = macLinks.intel;
    link.download = "";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
});

const scrollToSection = () => {
  nextTick(() => {
    window.scrollTo({
      top: downloadSection.value.offsetTop,
      behavior: "smooth",
    });
  });
};

const title = "Download the Modrinth App!";
const description =
  "The Modrinth App is a unique, open source launcher that allows you to play your favorite mods, and keep them up to date, all in one neat little package.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});
</script>

<template>
  <div>
    <div class="landing-hero">
      <h1 class="main-header">
        Download Modrinth <br v-if="os" />
        App
        {{ os ? `for ${os}` : "" }}
      </h1>
      <h2 class="main-subheader">
        The Modrinth App is a unique, open source launcher that allows you to play your favorite
        mods, and keep them up to date, all in one neat little package.
      </h2>
      <div class="button-group">
        <button
          v-if="os"
          class="iconified-button brand-button btn btn-large"
          rel="noopener nofollow"
          @click="downloadLauncher"
        >
          <svg
            v-if="os === 'Linux'"
            class="light-icon"
            width="45"
            height="46"
            viewBox="0 0 45 46"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              id="Subtract"
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M23.5477 0H23.5461C23.2284 0 22.9077 0.0163711 22.5876 0.0415902C20.4638 0.208942 18.8991 0.910133 17.7996 2.0082C16.7124 3.09404 16.1751 4.47323 15.9172 5.82868C15.6602 7.18017 15.6685 8.57463 15.7244 9.76049C15.7464 10.2267 15.7737 10.6424 15.7981 11.0129L15.7981 11.0129C15.8393 11.6392 15.8719 12.1361 15.856 12.5288C15.7269 14.3647 15.3652 15.7513 14.1358 17.5116C12.4894 19.4715 10.1353 22.6851 9.00147 26.0938C8.50987 27.5652 8.23247 29.1259 8.37804 30.6595C8.14194 30.9205 7.94151 31.1909 7.77396 31.4209L7.69935 31.5235C7.53577 31.7486 7.41036 31.9212 7.28343 32.0672C7.22885 32.1165 7.13829 32.1747 6.95723 32.2529C6.85646 32.2965 6.74665 32.3386 6.60714 32.3918L6.60229 32.3937L6.60063 32.3943C6.4689 32.4446 6.31402 32.5038 6.15349 32.5722L6.14713 32.575C6.11461 32.5891 6.08029 32.6038 6.04439 32.6191L6.04194 32.6201C5.49857 32.8521 4.5956 33.2374 4.06765 34.2907L4.06313 34.2998L4.05878 34.3089C3.83111 34.787 3.71086 35.3126 3.71965 35.8518C3.72044 36.2784 3.77526 36.6973 3.83488 36.9965L3.8573 37.1494C3.90153 37.4502 3.93607 37.685 3.9521 37.8975C3.96952 38.1284 3.9557 38.2416 3.94333 38.2911C3.47501 39.5811 3.29341 40.7747 3.80981 41.7808L3.81261 41.7862C4.34311 42.8045 5.41437 43.1509 6.09597 43.3713L6.14873 43.3884L6.1831 43.3968C6.76559 43.5407 7.45296 43.6219 8.08651 43.6967L8.08657 43.6967C8.33286 43.7258 8.57102 43.7539 8.79172 43.7844C9.65223 43.9032 10.4057 44.0681 11.0346 44.4066L11.0467 44.4131L11.0589 44.4193C12.8487 45.3199 14.8309 45.8073 16.5413 45.3594C17.5947 45.1214 18.5098 44.4944 19.1114 43.6161C19.5314 43.5681 19.9501 43.481 20.3513 43.3976L20.4887 43.3691C21.1456 43.2333 21.8552 43.0939 22.7255 43.039L22.7353 43.0384L22.7452 43.0376C23.2171 42.9984 23.7676 43.0823 24.5478 43.2012L24.6654 43.2192C25.2572 43.3092 25.9398 43.4072 26.6797 43.4223L26.7366 43.5729L26.7865 43.6228C27.7132 45.3272 29.3865 46.1223 31.1344 45.9841C32.8861 45.8473 34.6161 44.7911 35.9501 43.2465L35.9575 43.2379L35.9647 43.2291C36.4178 42.6799 37.0511 42.2572 37.7983 41.8603C38.1016 41.6992 38.4052 41.5515 38.7175 41.3996L38.7179 41.3994L38.936 41.2932C39.3111 41.1099 39.7022 40.9143 40.0596 40.6996C40.783 40.2838 41.6765 39.5491 41.7413 38.3282C41.8014 37.2663 41.2531 36.3256 40.4338 35.3719V35.1588L40.1744 34.8993C40.1374 34.8477 40.0795 34.7411 40.0113 34.5353C39.934 34.302 39.8672 34.0188 39.7856 33.6562C39.6368 32.9545 39.4127 31.9102 38.5493 31.186L38.3995 31.0604C38.9576 28.7316 38.5908 26.4633 37.8259 24.46C36.7898 21.7201 34.9984 19.3692 33.6785 17.7973L33.6778 17.7965C32.1849 15.9116 31.019 14.4397 31.0434 12.2669C31.0453 12.1131 31.0478 11.9522 31.0504 11.7849C31.0801 9.87812 31.1226 7.15021 30.3612 4.83914C29.9387 3.55665 29.2488 2.33019 28.115 1.42642C26.9706 0.514298 25.4646 0.00304389 23.5477 0ZM22.666 1.0385C22.9685 1.01467 23.2618 1 23.546 1C30.2102 1.01058 30.1098 7.76032 30.0499 11.7866C30.0475 11.9472 30.0452 12.1035 30.0433 12.2548C30.0146 14.7849 31.382 16.51 32.8074 18.3084C32.8393 18.3487 32.8713 18.3891 32.9033 18.4295C34.205 19.9787 35.9137 22.23 36.8908 24.815C37.692 26.9123 37.9982 29.2443 37.208 31.5873C37.3329 31.5924 37.4538 31.6331 37.5563 31.7047C37.6223 31.7737 37.6877 31.8118 37.7513 31.8488C37.8026 31.8787 37.8527 31.908 37.901 31.9522H37.9065C38.4748 32.4288 38.6527 33.1347 38.8085 33.8698C38.9698 34.5867 39.1165 35.2008 39.4282 35.5675L39.4337 35.573V35.7508C40.376 36.7867 40.7848 37.5402 40.7427 38.2735C40.706 38.9775 40.1908 39.4725 39.5528 39.8373C39.1757 40.0647 38.7412 40.276 38.2863 40.4973C37.2045 41.0234 36.0076 41.6055 35.1932 42.5928C33.974 44.0045 32.4688 44.8772 31.0553 44.9872C29.6418 45.099 28.3182 44.45 27.6013 43.0237L27.5958 43.0182C27.5697 42.949 27.5454 42.89 27.523 42.8356C27.4653 42.6951 27.4199 42.5847 27.3868 42.4077C26.4008 42.4735 25.4823 42.3328 24.6448 42.2044C23.9204 42.0934 23.2567 41.9917 22.6623 42.041C21.6653 42.1039 20.8661 42.2696 20.1753 42.4127C19.5578 42.5407 19.027 42.6507 18.519 42.6533C18.0827 43.537 17.2687 44.175 16.3043 44.3877C14.9293 44.7543 13.206 44.3803 11.5083 43.526C10.4773 42.9711 9.26303 42.8279 8.13135 42.6944C7.52434 42.6228 6.9411 42.554 6.42267 42.426C5.68017 42.1858 5.01833 41.9365 4.69933 41.3242C4.38033 40.7027 4.43716 39.8502 4.89366 38.6035C5.02207 38.1946 4.94184 37.6503 4.84524 36.995C4.83702 36.9392 4.82868 36.8826 4.82033 36.8252C4.769 36.5758 4.7195 36.2073 4.7195 35.8425C4.71216 35.4612 4.7965 35.0853 4.9615 34.7388C5.32126 34.021 5.91228 33.7658 6.46354 33.5276C6.49097 33.5158 6.5183 33.504 6.5455 33.4922C6.68913 33.4309 6.82976 33.3772 6.96561 33.3253C7.37077 33.1706 7.73344 33.0321 8.00667 32.7588C8.17742 32.5672 8.34012 32.343 8.50901 32.1103C8.72634 31.8108 8.95394 31.4971 9.22217 31.2207C9.27154 31.1251 9.34021 31.0408 9.42383 30.9732C9.19833 29.4973 9.44033 27.9353 9.95 26.41C11.0298 23.1632 13.3068 20.0483 14.9293 18.1215C16.3043 16.1653 16.715 14.5868 16.8543 12.5848C16.8738 12.1381 16.8352 11.5454 16.7908 10.8636C16.5636 7.37332 16.1844 1.54924 22.666 1.0385ZM24.5397 7.2425H24.5158C24.1226 7.24289 23.7402 7.3698 23.4268 7.60733C23.065 7.83891 22.7883 8.18188 22.6385 8.5845C22.4431 9.02748 22.3491 9.50854 22.3635 9.9925V10.0292C22.367 10.1519 22.378 10.2744 22.3965 10.3958C22.582 10.4694 22.8174 10.5436 23.0524 10.6176C23.2099 10.6673 23.3673 10.7169 23.5093 10.7662C23.4873 10.6048 23.4763 10.4398 23.4727 10.2767V10.2418C23.4845 9.98963 23.5403 9.74144 23.6377 9.5085C23.7102 9.27639 23.8394 9.06596 24.0135 8.89617C24.1712 8.70917 24.3233 8.65417 24.4938 8.65417H24.5232C24.7047 8.65417 24.8557 8.77281 25.0081 8.89261L25.0127 8.89617C25.0488 8.95476 25.0822 9.00626 25.1132 9.0541C25.2112 9.20535 25.2855 9.32003 25.3482 9.50667C25.4263 9.7431 25.4641 9.99101 25.46 10.24V10.2767C25.449 10.5297 25.3958 10.779 25.2987 11.0137C25.2573 11.0863 25.1757 11.1644 25.0976 11.2393C25.0476 11.2872 24.9989 11.3337 24.9632 11.3767C25.1009 11.4051 25.2359 11.4456 25.3665 11.4977C25.4774 11.5867 25.5956 11.6303 25.7223 11.6771C25.7757 11.6968 25.8306 11.717 25.8872 11.7415C25.9426 11.7634 25.9966 11.7892 26.0485 11.8185C26.2203 11.6435 26.3534 11.4342 26.439 11.2043C26.6168 10.7958 26.7103 10.3555 26.714 9.91L26.7213 9.954C26.7222 9.96709 26.7246 9.98002 26.7287 9.9925V9.8C26.7287 9.81833 26.7259 9.83667 26.7232 9.855C26.7204 9.87333 26.7177 9.89167 26.7177 9.91C26.703 9.42417 26.6058 9.0575 26.4133 8.58267C26.2153 8.21417 25.9587 7.853 25.6103 7.6055C25.2657 7.35617 24.9302 7.2425 24.5397 7.2425ZM19.1515 7.35067H19.0855C18.7958 7.35067 18.5538 7.4735 18.3192 7.71733C18.0782 8.00351 17.9013 8.33795 17.8003 8.69817C17.6731 9.09315 17.6233 9.50898 17.6537 9.92283C17.668 10.345 17.7645 10.7604 17.9378 11.1457C18.0826 11.4065 18.2514 11.6532 18.442 11.8827C18.5311 11.8615 18.6136 11.8187 18.6822 11.758C18.748 11.6843 18.8058 11.6467 18.8698 11.6049C18.9109 11.578 18.9547 11.5495 19.0048 11.5087C18.9792 11.4977 18.9608 11.4812 18.9425 11.4647C18.8178 11.34 18.7225 11.219 18.6382 10.9733C18.6295 10.9464 18.6213 10.9211 18.6135 10.8969C18.5496 10.7 18.5111 10.5814 18.4915 10.361V10.3335C18.4698 10.1309 18.4841 9.92611 18.5337 9.7285C18.5607 9.54657 18.6367 9.37544 18.7537 9.2335C18.8563 9.06483 18.9645 9.0025 19.0947 8.9915H19.1332C19.1941 8.98752 19.2552 8.99587 19.3129 9.01602C19.3705 9.03618 19.4235 9.06775 19.4687 9.10883C19.6109 9.24277 19.7158 9.41148 19.773 9.59833C19.7858 9.63881 19.7979 9.67596 19.8092 9.71087C19.8662 9.88628 19.905 10.0055 19.9233 10.2088V10.2363C19.9417 10.3958 19.9398 10.5608 19.9178 10.7258C19.9701 10.6973 20.0226 10.6672 20.0762 10.6365C20.2463 10.5391 20.4264 10.4359 20.6383 10.3592C20.6632 10.3536 20.6875 10.3458 20.7126 10.3378C20.7375 10.3298 20.7631 10.3216 20.7905 10.3152V10.1685C20.8052 10.0475 20.807 9.9265 20.7942 9.68083V9.6735C20.7703 9.184 20.6787 8.8155 20.5137 8.45067C20.367 8.12617 20.1507 7.83467 19.883 7.59817C19.6465 7.43867 19.4118 7.35067 19.1515 7.35067ZM23.8265 11.2098C22.941 10.5975 22.2058 10.4783 21.5972 10.4783H21.599C21.2837 10.4783 20.9995 10.5315 20.7447 10.5975C20.0058 10.8377 19.5163 11.329 19.2065 11.6993V11.7048H19.2028C19.1423 11.758 19.0708 11.8093 18.8802 11.945C18.8088 11.99 18.7236 12.0537 18.623 12.1287C18.4574 12.2523 18.2501 12.407 17.9947 12.561C17.6317 12.7993 17.5125 13.1697 17.6408 13.661C17.7692 14.024 18.1652 14.5117 18.8985 14.9627H18.9077V14.9718C19.1865 15.1208 19.4099 15.3135 19.6231 15.4973C19.7589 15.6145 19.8905 15.728 20.0297 15.8243C20.2199 15.9557 20.4169 16.0769 20.62 16.1873C20.8828 16.2829 21.1621 16.3246 21.4413 16.3102C22.2045 16.3552 22.7698 16.0785 23.269 15.8342L23.2967 15.8207C23.5576 15.695 23.7982 15.5366 24.0381 15.3788C24.2601 15.2327 24.4814 15.0871 24.7175 14.9682V14.9645C25.7203 14.5997 26.4408 13.9892 26.6663 13.4208C26.7229 13.2834 26.7496 13.1354 26.7445 12.9868C26.7394 12.8383 26.7028 12.6925 26.637 12.5592V12.7993C26.5032 12.5555 26.2282 12.3172 25.7607 12.0678H25.7552C25.6086 12.0127 25.4746 11.963 25.3509 11.9171C24.6553 11.6593 24.2825 11.5211 23.8265 11.2098ZM29.9113 22.6058C28.9232 20.7762 27.3868 17.0032 26.7305 14.4053H26.7287C26.3455 14.9517 25.6708 15.4412 24.8147 15.6887H24.8055C24.5824 15.7556 24.3538 15.8976 24.104 16.0528C23.9021 16.1782 23.6865 16.3122 23.4488 16.422C22.9593 16.6677 22.3763 16.9133 21.621 16.9133C21.5616 16.9133 21.5005 16.9086 21.435 16.9035L21.4193 16.9023C20.6878 16.9023 20.1983 16.6567 19.8133 16.2882C19.626 16.1704 19.4595 16.0231 19.2918 15.8748C19.115 15.7184 18.9368 15.5608 18.7317 15.4357C18.6433 15.3534 18.4753 15.2449 18.3474 15.1624C18.2843 15.1216 18.2309 15.0872 18.2018 15.0653C18.0598 17.7587 16.5369 21.0396 15.4344 23.4147C15.2131 23.8914 15.0087 24.3316 14.8358 24.7233C14.1222 26.3938 13.7202 28.1808 13.6497 29.996C11.7833 27.5082 13.1473 24.3072 13.8733 22.9303C14.6892 21.3922 14.8175 21.0255 14.6213 21.1447C13.8862 22.3418 12.7348 24.2485 12.2857 26.2102C12.051 27.2368 12.0107 28.247 12.3132 29.226C12.6138 30.1903 13.2665 31.052 14.4215 31.7853C15.9927 32.7497 17.067 33.7177 17.6995 34.5738C18.332 35.4282 18.53 36.2165 18.332 36.706C18.2262 36.9767 18.02 37.196 17.7563 37.3183C17.5235 37.3862 17.2375 37.4412 16.9038 37.4412C17.1014 37.6184 17.2714 37.8241 17.408 38.0517C17.694 38.345 17.9416 38.6733 18.145 39.0288C21.3863 41.2362 25.1997 40.3818 27.9497 38.4165C27.9605 38.3792 27.9714 38.3419 27.9822 38.3047C28.2443 37.4049 28.4993 36.5295 28.5345 35.9415V35.936C28.5987 34.6288 28.672 33.4867 28.9103 32.5095C29.1505 31.5598 29.5758 30.8247 30.3642 30.3333C30.4616 30.318 30.56 30.2827 30.6563 30.2481C30.6869 30.2371 30.7173 30.2262 30.7473 30.216C30.7558 30.0996 30.7642 30.0546 30.7769 29.9868C30.778 29.9809 30.7792 29.9748 30.7803 29.9685C31.0077 28.5 32.3478 28.3827 34.0253 29.116C35.6442 29.8475 36.2547 30.5038 35.9778 31.36H36.0072C36.0438 31.36 36.0805 31.3594 36.1172 31.3588C36.1905 31.3576 36.2638 31.3563 36.3372 31.36V31.4388C36.6727 30.3407 35.9742 29.6 34.0822 28.621C33.9694 28.6097 33.8595 28.5631 33.7512 28.5171C33.7362 28.5108 33.7213 28.5044 33.7063 28.4982C33.9612 27.4605 33.9062 26.4173 33.6678 25.4365C33.1527 23.264 31.7318 21.361 30.6502 20.382C30.4503 20.382 30.4723 20.6295 30.8757 20.9962C31.8712 21.911 34.0565 25.2037 32.8722 28.2672C32.5367 28.1828 32.2177 28.1407 31.9317 28.1498C31.4788 25.6473 30.4357 23.5848 29.9113 22.6058ZM32.3533 33.3528C31.3652 33.1108 30.8042 31.9283 30.7363 30.8283V30.832L30.6538 30.8723C30.0433 31.2152 29.6987 31.8238 29.4787 32.6763C29.2605 33.5545 29.189 34.693 29.1212 36.002C29.0756 36.7223 28.8292 37.5563 28.5746 38.4181C28.4709 38.7694 28.3658 39.1252 28.2723 39.4798C27.9497 40.699 27.7847 41.8448 28.1697 42.5782L28.1788 42.5837C28.7967 43.8633 29.8472 44.3272 31.0645 44.23C32.2855 44.1365 33.6642 43.3977 34.7972 42.0465C35.7047 40.9513 36.9722 40.3422 38.0484 39.8251C38.5178 39.5996 38.9508 39.3915 39.3017 39.1682C39.8755 38.8033 40.1872 38.5595 40.2147 38.0792L40.2202 38.0993V38.0663L40.2147 38.0792C40.2312 37.6502 39.9397 36.9553 39.0487 35.9892C38.6065 35.4932 38.4694 34.8431 38.324 34.1536C38.3156 34.1142 38.3073 34.0745 38.2988 34.0348C38.1412 33.4005 37.9633 32.7937 37.582 32.4288L37.5765 32.4233C37.0815 31.9503 36.6012 31.9192 36.0237 31.932L35.6368 31.9503C34.9823 32.7423 33.4313 33.5985 32.3533 33.3528ZM10.929 30.9182H10.9107C10.3918 30.9182 10.0435 31.1657 9.69517 31.5342C9.5004 31.7412 9.31978 31.9881 9.13328 32.2431C8.92831 32.5233 8.71624 32.8132 8.4705 33.0705V33.0742H8.46683C8.1027 33.429 7.67066 33.5917 7.25569 33.7479C7.0961 33.808 6.93904 33.8671 6.78933 33.9358C6.72367 33.9655 6.65958 33.9934 6.59713 34.0206C6.14698 34.2164 5.78193 34.3753 5.52433 34.913C5.29333 35.3347 5.34467 35.9012 5.4455 36.6363C5.45614 36.7047 5.46735 36.774 5.47868 36.844C5.57861 37.4618 5.68765 38.136 5.495 38.686L5.49133 38.6915V38.6988C5.05867 39.8502 5.06417 40.5872 5.26583 40.9538C5.473 41.3205 5.91483 41.5662 6.59133 41.6872C6.96665 41.7547 7.39979 41.7941 7.86764 41.8367C9.08866 41.9478 10.5461 42.0804 11.8292 42.7908C13.4333 43.6452 15.0375 43.889 16.22 43.6415C17.4025 43.3738 18.178 42.6332 18.2825 41.2692V41.2582C18.3577 40.4533 17.7417 39.4762 16.946 38.3798C16.7146 38.075 16.4714 37.7766 16.2319 37.4828C15.6462 36.764 15.0824 36.0723 14.7662 35.3787L14.7607 35.3732L13.0923 32.3225C12.5112 31.5543 11.9062 31.0447 11.2168 30.9438C11.1215 30.9273 11.0262 30.9182 10.929 30.9182ZM23.5083 14.5427C23.6897 14.4504 23.8657 14.3609 24.0355 14.2825L24.0337 14.2843C24.4698 14.0202 24.892 13.7339 25.2987 13.4263C25.6305 13.0633 25.8028 12.847 25.5682 12.8177C25.4293 12.8177 25.3757 12.8992 25.2985 13.0166C25.2433 13.1007 25.176 13.2031 25.0567 13.3072C24.8433 13.4469 24.6017 13.6265 24.3669 13.8009C24.1922 13.9307 24.0212 14.0577 23.8687 14.1633C23.2362 14.5282 22.1948 15.0177 21.3057 15.0177C20.4161 15.0177 19.7068 14.5318 19.1721 14.1656L19.1662 14.1615C19.0678 14.0713 18.976 13.9809 18.8905 13.8966C18.7447 13.753 18.6171 13.6272 18.5062 13.551C18.4612 13.5086 18.4274 13.4544 18.394 13.4007C18.332 13.3013 18.2711 13.2034 18.1413 13.1843C18.013 13.1843 17.9763 13.5492 18.277 13.7948C18.3752 13.8656 18.4936 13.9772 18.6302 14.106C18.7307 14.2008 18.8412 14.3049 18.9608 14.409C19.5292 14.7738 20.3138 15.2633 21.3075 15.2633C22.0911 15.2633 22.8372 14.8839 23.5083 14.5427ZM22.0518 11.34V11.3803C22.0735 11.4531 22.1637 11.4663 22.2486 11.4788C22.2856 11.4842 22.3217 11.4895 22.3507 11.4995C22.3795 11.5142 22.4061 11.5324 22.4323 11.5502C22.4875 11.5879 22.5407 11.6242 22.6092 11.6242C22.7008 11.6242 22.8457 11.593 22.8567 11.5032C22.875 11.3785 22.6972 11.2557 22.5817 11.2557C22.435 11.1988 22.2407 11.1695 22.105 11.2483C22.0757 11.263 22.0408 11.3033 22.0518 11.34ZM20.8356 11.482C20.9223 11.4694 21.0157 11.4557 21.0417 11.3803V11.3767H21.0362V11.34C21.0472 11.3033 21.016 11.2648 20.9812 11.2483C20.8437 11.1713 20.653 11.2007 20.5063 11.2575C20.3927 11.2575 20.213 11.3803 20.2313 11.5013C20.2423 11.5893 20.389 11.626 20.4807 11.626C20.5494 11.626 20.6048 11.588 20.6612 11.5495C20.686 11.5325 20.711 11.5154 20.7373 11.5013C20.7656 11.4923 20.8 11.4872 20.8356 11.482Z"
              fill="white"
            />
          </svg>
          <svg
            v-if="os === 'Linux'"
            class="dark-icon"
            role="img"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"
            />
          </svg>
          <svg
            v-else-if="os === 'Windows'"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 4875 4875"
            fill="currentColor"
          >
            <path
              d="M0 0h2311v2310H0zm2564 0h2311v2310H2564zM0 2564h2311v2311H0zm2564 0h2311v2311H2564"
            />
          </svg>
          <svg
            v-else-if="os === 'Mac'"
            xmlns="http://www.w3.org/2000/svg"
            width="44"
            height="44"
            viewBox="0 0 44 44"
            fill="none"
          >
            <path
              d="M33.5752 37.7889C31.3308 39.8664 28.8802 39.5384 26.5212 38.5543C24.0248 37.5484 21.7345 37.5046 19.1007 38.5543C15.8027 39.9102 14.0621 39.5165 12.0924 37.7889C0.915868 26.789 2.56487 10.0377 15.253 9.42536C18.3449 9.57844 20.4978 11.0436 22.3071 11.1748C25.0096 10.65 27.5976 9.14107 30.4834 9.33789C33.9417 9.60031 36.5526 10.9124 38.2703 13.2742C31.1247 17.3637 32.8195 26.3516 39.3697 28.8665C38.0642 32.1468 36.3694 35.4052 33.5523 37.8108L33.5752 37.7889ZM22.078 9.29415C21.7345 4.41745 25.8799 0.393635 30.6437 0C31.3079 5.6421 25.2844 9.84086 22.078 9.29415Z"
              fill="currentColor"
            />
          </svg>
          Download the Modrinth App
        </button>
        <button class="iconified-button outline-button btn btn-large" @click="scrollToSection">
          More Download Options
        </button>
      </div>
      <img src="https://cdn-raw.modrinth.com/app-landing/app-screenshot.webp" alt="cube maze" />
      <div class="bottom-transition" />
    </div>
    <div class="features">
      <h1 class="subheader">
        Unlike any launcher <br />
        you've used before
      </h1>
      <div class="feature-grid">
        <div class="feature gradient-border mods">
          <div class="search-bar">
            <h4>Installed mods</h4>
            <div class="mini-input">
              <SearchIcon />
              <div class="search">Search mods</div>
            </div>
          </div>
          <div class="header row">
            <div />
            <div class="cell">Name</div>
            <div class="cell">Version</div>
            <div class="cell">Actions</div>
          </div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/P7dR8mSH/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Fabric API</div>
                <div class="description">by modmuss</div>
              </div>
              <div class="cell important">0.86.1+1.20.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/AANobbMI/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Sodium</div>
                <div class="description">by jellysquid3</div>
              </div>
              <div class="cell">mc1.20.1-0.5.0</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/YL57xq9U/dc558eece920db435f9823ce86de0c4cde89800b.png"
                />
              </div>
              <div class="cell">
                <div class="name">Iris Shaders</div>
                <div class="description">by coderbot</div>
              </div>
              <div class="cell">1.6.5+1.20.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/gvQqBUqZ/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Lithium</div>
                <div class="description">by jellysquid3</div>
              </div>
              <div class="cell">mc1.20.1-0.11.2</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/mOgUt4GM/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Mod Menu</div>
                <div class="description">by Prospector</div>
              </div>
              <div class="cell">7.2.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/9s6osm5g/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Cloth Config API</div>
                <div class="description">by shedaniel</div>
              </div>
              <div class="cell">11.1.106+fabric</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/lhGA9TYQ/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Architectury API</div>
                <div class="description">by shedaniel</div>
              </div>
              <div class="cell">9.1.12+fabric</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/nrJ2NpD0/0efcf28eb5c18bed0cc47d786879e32550861ca4.png"
                />
              </div>
              <div class="cell">
                <div class="name">Craftify</div>
                <div class="description">by ThatGravyBoat</div>
              </div>
              <div class="cell">8.5.2023</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
          </div>
          <h3>Mod management</h3>
          <p>
            Modrinth makes it easy to manage all your mods in one place. You can install, uninstall,
            and update mods with a single click.
          </p>
        </div>
        <div class="feature gradient-border playing">
          <div class="text">
            <h3>Play with your favorite mods</h3>
            <p>Use the Modrinth App to download and play with your favorite mods and modpacks.</p>
          </div>
          <img
            src="https://cdn-raw.modrinth.com/app-landing/cobblemon-launcher.webp"
            alt="The Modrinth App playing Cobblemon for Fabric"
            class="launcher"
          />
          <img
            src="https://cdn-raw.modrinth.com/app-landing/cobblemon.webp"
            alt="Minecraft playing Cobblemon for Fabric"
            class="minecraft"
          />
        </div>
        <div class="feature gradient-border sharing">
          <div class="row header">Included mods <EditIcon /></div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/3ufwT9JF/2a15f23b7ffa2d50fc6ae1c42029a728ce3e2847.jpeg"
                />
              </div>
              <div class="cell">Ad Astra</div>
              <div class="cell">
                <div class="description">Live long and prosper, Ad Astra!</div>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png"
                />
              </div>
              <div class="cell">Spirit</div>
              <div class="cell">
                <div class="description">
                  A mod exploring soul magic in tools, blocks, weapons and more.
                </div>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/MI1LWe93/d42fb7a69f1e7a86584fa1ed43520af98acec065.png"
                />
              </div>
              <div class="cell">Creeper Overhaul</div>
              <div class="cell">A mod which overhauls the vanilla creepers!</div>
            </div>
          </div>
          <h3>Share Modpacks</h3>
          <p>
            Build, share, and play modpacks with any of the thousands of mods and modpacks hosted
            here on Modrinth.
          </p>
          <div class="export-card">
            <Avatar
              src="https://cdn.modrinth.com/data/mY0lOQFc/81c6eff2b86220e12e62a4ad0d2f364a605c42c4.png"
            />
            <div class="info">
              <div class="exporting">
                <div class="tag"><BoxIcon /> Modpack</div>
                <div class="small-button">
                  Share
                  <SendIcon />
                </div>
              </div>
              <h4 class="name">All of Fabric | Orion</h4>
              <div class="author">by AK</div>
            </div>
          </div>
        </div>
        <div class="feature gradient-border performance">
          <div class="title">
            <h4>Activity monitor</h4>
            <Badge color="green" type="Good performance" />
          </div>
          <div class="header row">
            <div />
            <div class="cell">Process name</div>
            <div class="cell">% CPU</div>
            <div class="cell">RAM</div>
          </div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <div>
                  <div class="icon-logo modrinth">
                    <AnimatedLogo class="icon" />
                  </div>
                </div>
              </div>
              <div class="cell important">Modrinth App</div>
              <div class="cell important">Small</div>
              <div class="cell important">{{ "< 150 MB" }}</div>
            </div>
            <div class="row">
              <div class="cell">
                <div>
                  <div class="icon-logo"></div>
                </div>
              </div>
              <div class="cell">Google Chrome</div>
              <div class="cell">150%</div>
              <div class="cell">∞ MB</div>
            </div>
            <div class="row">
              <div class="cell">
                <div>
                  <div class="icon-logo"></div>
                </div>
              </div>
              <div class="cell">Discord</div>
              <div class="cell">1 billion %</div>
              <div class="cell">∞ * ∞ MB</div>
            </div>
          </div>
          <h3>Performant</h3>
          <p>
            The Modrinth App performs better than many of the leading mod managers, using just 150mb
            of RAM!
          </p>
        </div>
        <div class="feature gradient-border website">
          <div class="icon-logo">
            <AnimatedLogo class="icon" />
          </div>
          <div class="ellipsis" />
          <div class="projects-showcase">
            <div v-for="(row, index) in rows" :key="index" class="row">
              <div v-for="n in 2" :key="n" class="row__content" :class="{ offset: index % 2 }">
                <nuxt-link
                  v-for="project in row"
                  :key="project.id"
                  class="project button-animation gradient-border"
                  :to="`/${project.project_type}/${project.slug ? project.slug : project.id}`"
                >
                  <Avatar :src="project.icon_url" :alt="project.title" size="sm" loading="lazy" />
                  <div class="project-info">
                    <span class="title">
                      {{ project.title }}
                    </span>
                    <span class="description">
                      {{ project.description }}
                    </span>
                  </div>
                </nuxt-link>
              </div>
            </div>
          </div>
          <h3>Website Integration</h3>
          <p>
            The Modrinth App is fully integrated with the website, so you can access all your
            favorite projects from the app!
          </p>
        </div>
        <div class="feature gradient-border importing">
          <div class="text">
            <h3>Profile importing</h3>
            <p>
              Import all your favorite profiles from the launcher you were using before, and get
              started with the Modrinth App in seconds!
            </p>
          </div>
          <div class="inner-ring ring">
            <div class="icon-logo">
              <AnimatedLogo class="icon" />
            </div>
            <div class="launcher-badge top-left">
              <img src="~/assets/images/external/gdlauncher.png" alt="GDLauncher" />
            </div>
            <div class="launcher-badge top-right">
              <img src="~/assets/images/external/multimc.webp" alt="MultiMC" />
            </div>
            <div class="launcher-badge bottom-left">
              <PrismIcon />
            </div>
            <div class="launcher-badge bottom-right">
              <ATLauncher />
            </div>
            <div class="launcher-badge bottom-middle">
              <CurseForge />
            </div>
            <div class="first-ring" />
            <div class="second-ring" />
            <div class="third-ring" />
          </div>
        </div>
      </div>
      <div class="feature-row">
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M21 8.25C21 8.70312 20.9258 9.13281 20.7773 9.53906C20.6289 9.94531 20.4141 10.3125 20.1328 10.6406C19.8516 10.9688 19.5273 11.2461 19.1602 11.4727C18.793 11.6992 18.3828 11.8516 17.9297 11.9297C17.8438 12.375 17.6914 12.7852 17.4727 13.1602C17.2539 13.5352 16.9766 13.8594 16.6406 14.1328C16.3047 14.4062 15.9375 14.6172 15.5391 14.7656C15.1406 14.9141 14.7109 14.9922 14.25 15H9.75C9.5 15 9.26172 15.0391 9.03516 15.1172C8.80859 15.1953 8.60156 15.3047 8.41406 15.4453C8.22656 15.5859 8.0625 15.7539 7.92188 15.9492C7.78125 16.1445 7.67578 16.3633 7.60547 16.6055C8.02734 16.707 8.41406 16.8711 8.76562 17.0977C9.11719 17.3242 9.42188 17.6016 9.67969 17.9297C9.9375 18.2578 10.1367 18.6172 10.2773 19.0078C10.418 19.3984 10.4922 19.8125 10.5 20.25C10.5 20.7656 10.4023 21.25 10.207 21.7031C10.0117 22.1562 9.74219 22.5547 9.39844 22.8984C9.05469 23.2422 8.65625 23.5117 8.20312 23.707C7.75 23.9023 7.26562 24 6.75 24C6.23438 24 5.75 23.9023 5.29688 23.707C4.84375 23.5117 4.44531 23.2461 4.10156 22.9102C3.75781 22.5742 3.48828 22.1758 3.29297 21.7148C3.09766 21.2539 3 20.7656 3 20.25C3 19.8047 3.07422 19.3789 3.22266 18.9727C3.37109 18.5664 3.57812 18.2031 3.84375 17.8828C4.10938 17.5625 4.42578 17.2852 4.79297 17.0508C5.16016 16.8164 5.5625 16.6602 6 16.582V7.41797C5.5625 7.33203 5.16016 7.17578 4.79297 6.94922C4.42578 6.72266 4.10938 6.44922 3.84375 6.12891C3.57812 5.80859 3.37109 5.44141 3.22266 5.02734C3.07422 4.61328 3 4.1875 3 3.75C3 3.23438 3.09766 2.75 3.29297 2.29688C3.48828 1.84375 3.75391 1.44922 4.08984 1.11328C4.42578 0.777344 4.82422 0.507812 5.28516 0.304688C5.74609 0.101562 6.23438 0 6.75 0C7.26562 0 7.75 0.0976563 8.20312 0.292969C8.65625 0.488281 9.05078 0.757812 9.38672 1.10156C9.72266 1.44531 9.99219 1.84375 10.1953 2.29688C10.3984 2.75 10.5 3.23438 10.5 3.75C10.5 4.19531 10.4258 4.62109 10.2773 5.02734C10.1289 5.43359 9.92188 5.79688 9.65625 6.11719C9.39062 6.4375 9.07422 6.71484 8.70703 6.94922C8.33984 7.18359 7.9375 7.33984 7.5 7.41797V14.2734C7.82812 14.0234 8.18359 13.832 8.56641 13.6992C8.94922 13.5664 9.34375 13.5 9.75 13.5H14.25C14.5 13.5 14.7383 13.4609 14.9648 13.3828C15.1914 13.3047 15.3984 13.1953 15.5859 13.0547C15.7734 12.9141 15.9375 12.7461 16.0781 12.5508C16.2188 12.3555 16.3242 12.1367 16.3945 11.8945C15.9727 11.793 15.5859 11.6289 15.2344 11.4023C14.8828 11.1758 14.5781 10.9023 14.3203 10.582C14.0625 10.2617 13.8633 9.90234 13.7227 9.50391C13.582 9.10547 13.5078 8.6875 13.5 8.25C13.5 7.73438 13.5977 7.25 13.793 6.79688C13.9883 6.34375 14.2539 5.94922 14.5898 5.61328C14.9258 5.27734 15.3242 5.00781 15.7852 4.80469C16.2461 4.60156 16.7344 4.5 17.25 4.5C17.7656 4.5 18.25 4.59766 18.7031 4.79297C19.1562 4.98828 19.5508 5.25781 19.8867 5.60156C20.2227 5.94531 20.4922 6.34375 20.6953 6.79688C20.8984 7.25 21 7.73438 21 8.25ZM4.5 3.75C4.5 4.0625 4.55859 4.35547 4.67578 4.62891C4.79297 4.90234 4.95312 5.14062 5.15625 5.34375C5.35938 5.54688 5.59766 5.70703 5.87109 5.82422C6.14453 5.94141 6.4375 6 6.75 6C7.0625 6 7.35547 5.94141 7.62891 5.82422C7.90234 5.70703 8.14062 5.54688 8.34375 5.34375C8.54688 5.14062 8.70703 4.90234 8.82422 4.62891C8.94141 4.35547 9 4.0625 9 3.75C9 3.4375 8.94141 3.14453 8.82422 2.87109C8.70703 2.59766 8.54688 2.35938 8.34375 2.15625C8.14062 1.95312 7.90234 1.79297 7.62891 1.67578C7.35547 1.55859 7.0625 1.5 6.75 1.5C6.4375 1.5 6.14453 1.55859 5.87109 1.67578C5.59766 1.79297 5.35938 1.95312 5.15625 2.15625C4.95312 2.35938 4.79297 2.59766 4.67578 2.87109C4.55859 3.14453 4.5 3.4375 4.5 3.75ZM9 20.25C9 19.9375 8.94141 19.6445 8.82422 19.3711C8.70703 19.0977 8.54688 18.8594 8.34375 18.6562C8.14062 18.4531 7.90234 18.293 7.62891 18.1758C7.35547 18.0586 7.0625 18 6.75 18C6.4375 18 6.14453 18.0586 5.87109 18.1758C5.59766 18.293 5.35938 18.4531 5.15625 18.6562C4.95312 18.8594 4.79297 19.0977 4.67578 19.3711C4.55859 19.6445 4.5 19.9375 4.5 20.25C4.5 20.5625 4.55859 20.8555 4.67578 21.1289C4.79297 21.4023 4.95312 21.6406 5.15625 21.8438C5.35938 22.0469 5.59766 22.207 5.87109 22.3242C6.14453 22.4414 6.4375 22.5 6.75 22.5C7.0625 22.5 7.35547 22.4414 7.62891 22.3242C7.90234 22.207 8.14062 22.0469 8.34375 21.8438C8.54688 21.6406 8.70703 21.4023 8.82422 21.1289C8.94141 20.8555 9 20.5625 9 20.25ZM17.25 10.5C17.5625 10.5 17.8555 10.4414 18.1289 10.3242C18.4023 10.207 18.6406 10.0469 18.8438 9.84375C19.0469 9.64062 19.207 9.40234 19.3242 9.12891C19.4414 8.85547 19.5 8.5625 19.5 8.25C19.5 7.9375 19.4414 7.64453 19.3242 7.37109C19.207 7.09766 19.0469 6.85938 18.8438 6.65625C18.6406 6.45312 18.4023 6.29297 18.1289 6.17578C17.8555 6.05859 17.5625 6 17.25 6C16.9375 6 16.6445 6.05859 16.3711 6.17578C16.0977 6.29297 15.8594 6.45312 15.6562 6.65625C15.4531 6.85938 15.293 7.09766 15.1758 7.37109C15.0586 7.64453 15 7.9375 15 8.25C15 8.5625 15.0586 8.85547 15.1758 9.12891C15.293 9.40234 15.4531 9.64062 15.6562 9.84375C15.8594 10.0469 16.0977 10.207 16.3711 10.3242C16.6445 10.4414 16.9375 10.5 17.25 10.5Z"
                fill="url(#paint0_linear_897_3796)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_897_3796"
                  x1="12"
                  y1="0"
                  x2="12"
                  y2="24"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M21 8.25C21 8.70312 20.9258 9.13281 20.7773 9.53906C20.6289 9.94531 20.4141 10.3125 20.1328 10.6406C19.8516 10.9688 19.5273 11.2461 19.1602 11.4727C18.793 11.6992 18.3828 11.8516 17.9297 11.9297C17.8438 12.375 17.6914 12.7852 17.4727 13.1602C17.2539 13.5352 16.9766 13.8594 16.6406 14.1328C16.3047 14.4062 15.9375 14.6172 15.5391 14.7656C15.1406 14.9141 14.7109 14.9922 14.25 15H9.75C9.5 15 9.26172 15.0391 9.03516 15.1172C8.80859 15.1953 8.60156 15.3047 8.41406 15.4453C8.22656 15.5859 8.0625 15.7539 7.92188 15.9492C7.78125 16.1445 7.67578 16.3633 7.60547 16.6055C8.02734 16.707 8.41406 16.8711 8.76562 17.0977C9.11719 17.3242 9.42188 17.6016 9.67969 17.9297C9.9375 18.2578 10.1367 18.6172 10.2773 19.0078C10.418 19.3984 10.4922 19.8125 10.5 20.25C10.5 20.7656 10.4023 21.25 10.207 21.7031C10.0117 22.1562 9.74219 22.5547 9.39844 22.8984C9.05469 23.2422 8.65625 23.5117 8.20312 23.707C7.75 23.9023 7.26562 24 6.75 24C6.23438 24 5.75 23.9023 5.29688 23.707C4.84375 23.5117 4.44531 23.2461 4.10156 22.9102C3.75781 22.5742 3.48828 22.1758 3.29297 21.7148C3.09766 21.2539 3 20.7656 3 20.25C3 19.8047 3.07422 19.3789 3.22266 18.9727C3.37109 18.5664 3.57812 18.2031 3.84375 17.8828C4.10938 17.5625 4.42578 17.2852 4.79297 17.0508C5.16016 16.8164 5.5625 16.6602 6 16.582V7.41797C5.5625 7.33203 5.16016 7.17578 4.79297 6.94922C4.42578 6.72266 4.10938 6.44922 3.84375 6.12891C3.57812 5.80859 3.37109 5.44141 3.22266 5.02734C3.07422 4.61328 3 4.1875 3 3.75C3 3.23438 3.09766 2.75 3.29297 2.29688C3.48828 1.84375 3.75391 1.44922 4.08984 1.11328C4.42578 0.777344 4.82422 0.507812 5.28516 0.304688C5.74609 0.101562 6.23438 0 6.75 0C7.26562 0 7.75 0.0976563 8.20312 0.292969C8.65625 0.488281 9.05078 0.757812 9.38672 1.10156C9.72266 1.44531 9.99219 1.84375 10.1953 2.29688C10.3984 2.75 10.5 3.23438 10.5 3.75C10.5 4.19531 10.4258 4.62109 10.2773 5.02734C10.1289 5.43359 9.92188 5.79688 9.65625 6.11719C9.39062 6.4375 9.07422 6.71484 8.70703 6.94922C8.33984 7.18359 7.9375 7.33984 7.5 7.41797V14.2734C7.82812 14.0234 8.18359 13.832 8.56641 13.6992C8.94922 13.5664 9.34375 13.5 9.75 13.5H14.25C14.5 13.5 14.7383 13.4609 14.9648 13.3828C15.1914 13.3047 15.3984 13.1953 15.5859 13.0547C15.7734 12.9141 15.9375 12.7461 16.0781 12.5508C16.2188 12.3555 16.3242 12.1367 16.3945 11.8945C15.9727 11.793 15.5859 11.6289 15.2344 11.4023C14.8828 11.1758 14.5781 10.9023 14.3203 10.582C14.0625 10.2617 13.8633 9.90234 13.7227 9.50391C13.582 9.10547 13.5078 8.6875 13.5 8.25C13.5 7.73438 13.5977 7.25 13.793 6.79688C13.9883 6.34375 14.2539 5.94922 14.5898 5.61328C14.9258 5.27734 15.3242 5.00781 15.7852 4.80469C16.2461 4.60156 16.7344 4.5 17.25 4.5C17.7656 4.5 18.25 4.59766 18.7031 4.79297C19.1562 4.98828 19.5508 5.25781 19.8867 5.60156C20.2227 5.94531 20.4922 6.34375 20.6953 6.79688C20.8984 7.25 21 7.73438 21 8.25ZM4.5 3.75C4.5 4.0625 4.55859 4.35547 4.67578 4.62891C4.79297 4.90234 4.95312 5.14062 5.15625 5.34375C5.35938 5.54688 5.59766 5.70703 5.87109 5.82422C6.14453 5.94141 6.4375 6 6.75 6C7.0625 6 7.35547 5.94141 7.62891 5.82422C7.90234 5.70703 8.14062 5.54688 8.34375 5.34375C8.54688 5.14062 8.70703 4.90234 8.82422 4.62891C8.94141 4.35547 9 4.0625 9 3.75C9 3.4375 8.94141 3.14453 8.82422 2.87109C8.70703 2.59766 8.54688 2.35938 8.34375 2.15625C8.14062 1.95312 7.90234 1.79297 7.62891 1.67578C7.35547 1.55859 7.0625 1.5 6.75 1.5C6.4375 1.5 6.14453 1.55859 5.87109 1.67578C5.59766 1.79297 5.35938 1.95312 5.15625 2.15625C4.95312 2.35938 4.79297 2.59766 4.67578 2.87109C4.55859 3.14453 4.5 3.4375 4.5 3.75ZM9 20.25C9 19.9375 8.94141 19.6445 8.82422 19.3711C8.70703 19.0977 8.54688 18.8594 8.34375 18.6562C8.14062 18.4531 7.90234 18.293 7.62891 18.1758C7.35547 18.0586 7.0625 18 6.75 18C6.4375 18 6.14453 18.0586 5.87109 18.1758C5.59766 18.293 5.35938 18.4531 5.15625 18.6562C4.95312 18.8594 4.79297 19.0977 4.67578 19.3711C4.55859 19.6445 4.5 19.9375 4.5 20.25C4.5 20.5625 4.55859 20.8555 4.67578 21.1289C4.79297 21.4023 4.95312 21.6406 5.15625 21.8438C5.35938 22.0469 5.59766 22.207 5.87109 22.3242C6.14453 22.4414 6.4375 22.5 6.75 22.5C7.0625 22.5 7.35547 22.4414 7.62891 22.3242C7.90234 22.207 8.14062 22.0469 8.34375 21.8438C8.54688 21.6406 8.70703 21.4023 8.82422 21.1289C8.94141 20.8555 9 20.5625 9 20.25ZM17.25 10.5C17.5625 10.5 17.8555 10.4414 18.1289 10.3242C18.4023 10.207 18.6406 10.0469 18.8438 9.84375C19.0469 9.64062 19.207 9.40234 19.3242 9.12891C19.4414 8.85547 19.5 8.5625 19.5 8.25C19.5 7.9375 19.4414 7.64453 19.3242 7.37109C19.207 7.09766 19.0469 6.85938 18.8438 6.65625C18.6406 6.45312 18.4023 6.29297 18.1289 6.17578C17.8555 6.05859 17.5625 6 17.25 6C16.9375 6 16.6445 6.05859 16.3711 6.17578C16.0977 6.29297 15.8594 6.45312 15.6562 6.65625C15.4531 6.85938 15.293 7.09766 15.1758 7.37109C15.0586 7.64453 15 7.9375 15 8.25C15 8.5625 15.0586 8.85547 15.1758 9.12891C15.293 9.40234 15.4531 9.64062 15.6562 9.84375C15.8594 10.0469 16.0977 10.207 16.3711 10.3242C16.6445 10.4414 16.9375 10.5 17.25 10.5Z"
                fill="url(#paint0_linear_944_4967)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4967"
                  x1="12"
                  y1="0"
                  x2="12"
                  y2="24"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Open source</h3>
          </div>
          <div class="description">
            Modrinth’s launcher is fully open source. You can view the source code on our
            <a href="https://github.com/modrinth/theseus" rel="noopener" :target="$external()"
              >GitHub</a
            >!
          </div>
        </div>
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <g clip-path="url(#clip0_897_3802)">
                <path
                  d="M4.39313 8.58984C2.31984 9.28359 0.75 10.9266 0.75 13.5C0.75 16.5938 3.28125 18.75 6.375 18.75H15.0173M21.9291 17.7066C22.7456 17.0297 23.25 16.013 23.25 14.625C23.25 11.8209 20.7656 10.605 18.75 10.5C18.3333 6.30281 15.4219 3.75 12 3.75C10.7738 3.75 9.71297 4.07484 8.83125 4.60031"
                  stroke="url(#paint0_linear_897_3802)"
                  stroke-width="2.34783"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
                <path
                  d="M21 21L3 3"
                  stroke="url(#paint1_linear_897_3802)"
                  stroke-width="2.34783"
                  stroke-miterlimit="10"
                  stroke-linecap="round"
                />
              </g>
              <defs>
                <linearGradient
                  id="paint0_linear_897_3802"
                  x1="12"
                  y1="3.75"
                  x2="12"
                  y2="18.75"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
                <linearGradient
                  id="paint1_linear_897_3802"
                  x1="12"
                  y1="3"
                  x2="12"
                  y2="21"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
                <clipPath id="clip0_897_3802">
                  <rect width="24" height="24" fill="white" />
                </clipPath>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="26"
              height="24"
              viewBox="0 0 26 24"
              fill="none"
            >
              <path
                d="M5.39313 8.58984C3.31984 9.28359 1.75 10.9266 1.75 13.5C1.75 16.5938 4.28125 18.75 7.375 18.75H16.0173M22.9291 17.7066C23.7456 17.0297 24.25 16.013 24.25 14.625C24.25 11.8209 21.7656 10.605 19.75 10.5C19.3333 6.30281 16.4219 3.75 13 3.75C11.7738 3.75 10.713 4.07484 9.83125 4.60031"
                stroke="url(#paint0_linear_944_4973)"
                stroke-width="2.34783"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M22 21L4 3"
                stroke="url(#paint1_linear_944_4973)"
                stroke-width="2.34783"
                stroke-miterlimit="10"
                stroke-linecap="round"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4973"
                  x1="13"
                  y1="3.75"
                  x2="13"
                  y2="18.75"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
                <linearGradient
                  id="paint1_linear_944_4973"
                  x1="13"
                  y1="3"
                  x2="13"
                  y2="21"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Offline mode</h3>
          </div>
          <div class="description">
            Play your mods, whether you are connected to the internet, or not.
          </div>
        </div>
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M22.8533 11.7089C22.8205 11.6379 22.0264 9.94977 20.2611 8.25801C17.9089 6.00383 14.938 4.8125 11.668 4.8125C8.39796 4.8125 5.42702 6.00383 3.07483 8.25801C1.30952 9.94977 0.51171 11.6406 0.482647 11.7089C0.440003 11.8008 0.417969 11.9003 0.417969 12.0009C0.417969 12.1015 0.440003 12.201 0.482647 12.2929C0.51546 12.3639 1.30952 14.0511 3.07483 15.7429C5.42702 17.9962 8.39796 19.1875 11.668 19.1875C14.938 19.1875 17.9089 17.9962 20.2611 15.7429C22.0264 14.0511 22.8205 12.3639 22.8533 12.2929C22.8959 12.201 22.918 12.1015 22.918 12.0009C22.918 11.9003 22.8959 11.8008 22.8533 11.7089ZM11.668 17.75C8.78234 17.75 6.2614 16.7446 4.17452 14.7627C3.31825 13.9466 2.58976 13.0161 2.01171 12C2.58961 10.9838 3.31811 10.0532 4.17452 9.2373C6.2614 7.25535 8.78234 6.25 11.668 6.25C14.5536 6.25 17.0745 7.25535 19.1614 9.2373C20.0193 10.053 20.7494 10.9836 21.3289 12C20.653 13.2093 17.7083 17.75 11.668 17.75ZM11.668 7.6875C10.7779 7.6875 9.90792 7.94042 9.16789 8.41429C8.42787 8.88815 7.8511 9.56167 7.5105 10.3497C7.16991 11.1377 7.08079 12.0048 7.25443 12.8413C7.42806 13.6779 7.85664 14.4463 8.48598 15.0494C9.11532 15.6525 9.91714 16.0632 10.7901 16.2296C11.663 16.396 12.5678 16.3106 13.39 15.9842C14.2123 15.6578 14.9151 15.1051 15.4096 14.3959C15.904 13.6867 16.168 12.8529 16.168 12C16.1667 10.8566 15.6922 9.76041 14.8486 8.95192C14.0049 8.14342 12.8611 7.68869 11.668 7.6875ZM11.668 14.875C11.0746 14.875 10.4946 14.7064 10.0012 14.3905C9.5079 14.0746 9.12338 13.6256 8.89632 13.1002C8.66926 12.5749 8.60985 11.9968 8.7256 11.4391C8.84136 10.8814 9.12708 10.3691 9.54664 9.96707C9.9662 9.56499 10.5007 9.29117 11.0827 9.18024C11.6646 9.06931 12.2678 9.12624 12.816 9.34385C13.3642 9.56145 13.8327 9.92994 14.1624 10.4027C14.492 10.8755 14.668 11.4314 14.668 12C14.668 12.7625 14.3519 13.4938 13.7893 14.0329C13.2267 14.5721 12.4636 14.875 11.668 14.875Z"
                fill="url(#paint0_linear_897_3809)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_897_3809"
                  x1="11.668"
                  y1="4.8125"
                  x2="11.668"
                  y2="19.1875"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M22.8533 11.7089C22.8205 11.6379 22.0264 9.94977 20.2611 8.25801C17.9089 6.00383 14.938 4.8125 11.668 4.8125C8.39796 4.8125 5.42702 6.00383 3.07483 8.25801C1.30952 9.94977 0.51171 11.6406 0.482647 11.7089C0.440003 11.8008 0.417969 11.9003 0.417969 12.0009C0.417969 12.1015 0.440003 12.201 0.482647 12.2929C0.51546 12.3639 1.30952 14.0511 3.07483 15.7429C5.42702 17.9962 8.39796 19.1875 11.668 19.1875C14.938 19.1875 17.9089 17.9962 20.2611 15.7429C22.0264 14.0511 22.8205 12.3639 22.8533 12.2929C22.8959 12.201 22.918 12.1015 22.918 12.0009C22.918 11.9003 22.8959 11.8008 22.8533 11.7089ZM11.668 17.75C8.78234 17.75 6.2614 16.7446 4.17452 14.7627C3.31825 13.9466 2.58976 13.0161 2.01171 12C2.58961 10.9838 3.31811 10.0532 4.17452 9.2373C6.2614 7.25535 8.78234 6.25 11.668 6.25C14.5536 6.25 17.0745 7.25535 19.1614 9.2373C20.0193 10.053 20.7494 10.9836 21.3289 12C20.653 13.2093 17.7083 17.75 11.668 17.75ZM11.668 7.6875C10.7779 7.6875 9.90792 7.94042 9.16789 8.41429C8.42787 8.88815 7.8511 9.56167 7.5105 10.3497C7.16991 11.1377 7.08079 12.0048 7.25443 12.8413C7.42806 13.6779 7.85664 14.4463 8.48598 15.0494C9.11532 15.6525 9.91714 16.0632 10.7901 16.2296C11.663 16.396 12.5678 16.3106 13.39 15.9842C14.2123 15.6578 14.9151 15.1051 15.4096 14.3959C15.904 13.6867 16.168 12.8529 16.168 12C16.1667 10.8566 15.6922 9.76041 14.8486 8.95192C14.0049 8.14342 12.8611 7.68869 11.668 7.6875ZM11.668 14.875C11.0746 14.875 10.4946 14.7064 10.0012 14.3905C9.5079 14.0746 9.12338 13.6256 8.89632 13.1002C8.66926 12.5749 8.60985 11.9968 8.7256 11.4391C8.84136 10.8814 9.12708 10.3691 9.54664 9.96707C9.9662 9.56499 10.5007 9.29117 11.0827 9.18024C11.6646 9.06931 12.2678 9.12624 12.816 9.34385C13.3642 9.56145 13.8327 9.92994 14.1624 10.4027C14.492 10.8755 14.668 11.4314 14.668 12C14.668 12.7625 14.3519 13.4938 13.7893 14.0329C13.2267 14.5721 12.4636 14.875 11.668 14.875Z"
                fill="url(#paint0_linear_944_4980)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4980"
                  x1="11.668"
                  y1="4.8125"
                  x2="11.668"
                  y2="19.1875"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Follow projects</h3>
          </div>
          <div class="description">Save content you love and receive updates with one click.</div>
        </div>
      </div>
    </div>
    <div ref="downloadSection" class="footer">
      <div class="section-badge">Download options</div>
      <div class="section-subheader">
        <div class="section-subheader-title">Download the Modrinth App</div>
        <div class="section-subheader-description">
          Our desktop app is available across all platforms, <br />
          choose your desired version.
        </div>
      </div>
      <div class="download-section">
        <div class="download-card">
          <div class="title">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4875 4875" fill="currentColor">
              <path
                d="M0 0h2311v2310H0zm2564 0h2311v2310H2564zM0 2564h2311v2311H0zm2564 0h2311v2311H2564"
              />
            </svg>
            Windows
          </div>
          <div class="description">
            <a ref="downloadWindows" :href="windowsLink" download="">
              <DownloadIcon />
              <span> Download the beta </span>
            </a>
          </div>
        </div>
        <div class="divider" />
        <div class="download-card">
          <div class="title">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="44"
              height="44"
              viewBox="0 0 44 44"
              fill="none"
            >
              <path
                d="M33.5752 37.7889C31.3308 39.8664 28.8802 39.5384 26.5212 38.5543C24.0248 37.5484 21.7345 37.5046 19.1007 38.5543C15.8027 39.9102 14.0621 39.5165 12.0924 37.7889C0.915868 26.789 2.56487 10.0377 15.253 9.42536C18.3449 9.57844 20.4978 11.0436 22.3071 11.1748C25.0096 10.65 27.5976 9.14107 30.4834 9.33789C33.9417 9.60031 36.5526 10.9124 38.2703 13.2742C31.1247 17.3637 32.8195 26.3516 39.3697 28.8665C38.0642 32.1468 36.3694 35.4052 33.5523 37.8108L33.5752 37.7889ZM22.078 9.29415C21.7345 4.41745 25.8799 0.393635 30.6437 0C31.3079 5.6421 25.2844 9.84086 22.078 9.29415Z"
                fill="currentColor"
              />
            </svg>
            Mac
          </div>
          <div class="description apple">
            <a :href="macLinks.appleSilicon" download="">
              <DownloadIcon />
              <span> Download for Apple Silicon </span>
            </a>
            <a :href="macLinks.intel" download="">
              <DownloadIcon />
              <span> Download for Intel </span>
            </a>
          </div>
        </div>
        <div class="divider" />
        <div class="download-card">
          <div class="title">
            <svg
              class="dark-icon"
              width="45"
              height="46"
              viewBox="0 0 45 46"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                id="Subtract"
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M23.5477 0H23.5461C23.2284 0 22.9077 0.0163711 22.5876 0.0415902C20.4638 0.208942 18.8991 0.910133 17.7996 2.0082C16.7124 3.09404 16.1751 4.47323 15.9172 5.82868C15.6602 7.18017 15.6685 8.57463 15.7244 9.76049C15.7464 10.2267 15.7737 10.6424 15.7981 11.0129L15.7981 11.0129C15.8393 11.6392 15.8719 12.1361 15.856 12.5288C15.7269 14.3647 15.3652 15.7513 14.1358 17.5116C12.4894 19.4715 10.1353 22.6851 9.00147 26.0938C8.50987 27.5652 8.23247 29.1259 8.37804 30.6595C8.14194 30.9205 7.94151 31.1909 7.77396 31.4209L7.69935 31.5235C7.53577 31.7486 7.41036 31.9212 7.28343 32.0672C7.22885 32.1165 7.13829 32.1747 6.95723 32.2529C6.85646 32.2965 6.74665 32.3386 6.60714 32.3918L6.60229 32.3937L6.60063 32.3943C6.4689 32.4446 6.31402 32.5038 6.15349 32.5722L6.14713 32.575C6.11461 32.5891 6.08029 32.6038 6.04439 32.6191L6.04194 32.6201C5.49857 32.8521 4.5956 33.2374 4.06765 34.2907L4.06313 34.2998L4.05878 34.3089C3.83111 34.787 3.71086 35.3126 3.71965 35.8518C3.72044 36.2784 3.77526 36.6973 3.83488 36.9965L3.8573 37.1494C3.90153 37.4502 3.93607 37.685 3.9521 37.8975C3.96952 38.1284 3.9557 38.2416 3.94333 38.2911C3.47501 39.5811 3.29341 40.7747 3.80981 41.7808L3.81261 41.7862C4.34311 42.8045 5.41437 43.1509 6.09597 43.3713L6.14873 43.3884L6.1831 43.3968C6.76559 43.5407 7.45296 43.6219 8.08651 43.6967L8.08657 43.6967C8.33286 43.7258 8.57102 43.7539 8.79172 43.7844C9.65223 43.9032 10.4057 44.0681 11.0346 44.4066L11.0467 44.4131L11.0589 44.4193C12.8487 45.3199 14.8309 45.8073 16.5413 45.3594C17.5947 45.1214 18.5098 44.4944 19.1114 43.6161C19.5314 43.5681 19.9501 43.481 20.3513 43.3976L20.4887 43.3691C21.1456 43.2333 21.8552 43.0939 22.7255 43.039L22.7353 43.0384L22.7452 43.0376C23.2171 42.9984 23.7676 43.0823 24.5478 43.2012L24.6654 43.2192C25.2572 43.3092 25.9398 43.4072 26.6797 43.4223L26.7366 43.5729L26.7865 43.6228C27.7132 45.3272 29.3865 46.1223 31.1344 45.9841C32.8861 45.8473 34.6161 44.7911 35.9501 43.2465L35.9575 43.2379L35.9647 43.2291C36.4178 42.6799 37.0511 42.2572 37.7983 41.8603C38.1016 41.6992 38.4052 41.5515 38.7175 41.3996L38.7179 41.3994L38.936 41.2932C39.3111 41.1099 39.7022 40.9143 40.0596 40.6996C40.783 40.2838 41.6765 39.5491 41.7413 38.3282C41.8014 37.2663 41.2531 36.3256 40.4338 35.3719V35.1588L40.1744 34.8993C40.1374 34.8477 40.0795 34.7411 40.0113 34.5353C39.934 34.302 39.8672 34.0188 39.7856 33.6562C39.6368 32.9545 39.4127 31.9102 38.5493 31.186L38.3995 31.0604C38.9576 28.7316 38.5908 26.4633 37.8259 24.46C36.7898 21.7201 34.9984 19.3692 33.6785 17.7973L33.6778 17.7965C32.1849 15.9116 31.019 14.4397 31.0434 12.2669C31.0453 12.1131 31.0478 11.9522 31.0504 11.7849C31.0801 9.87812 31.1226 7.15021 30.3612 4.83914C29.9387 3.55665 29.2488 2.33019 28.115 1.42642C26.9706 0.514298 25.4646 0.00304389 23.5477 0ZM22.666 1.0385C22.9685 1.01467 23.2618 1 23.546 1C30.2102 1.01058 30.1098 7.76032 30.0499 11.7866C30.0475 11.9472 30.0452 12.1035 30.0433 12.2548C30.0146 14.7849 31.382 16.51 32.8074 18.3084C32.8393 18.3487 32.8713 18.3891 32.9033 18.4295C34.205 19.9787 35.9137 22.23 36.8908 24.815C37.692 26.9123 37.9982 29.2443 37.208 31.5873C37.3329 31.5924 37.4538 31.6331 37.5563 31.7047C37.6223 31.7737 37.6877 31.8118 37.7513 31.8488C37.8026 31.8787 37.8527 31.908 37.901 31.9522H37.9065C38.4748 32.4288 38.6527 33.1347 38.8085 33.8698C38.9698 34.5867 39.1165 35.2008 39.4282 35.5675L39.4337 35.573V35.7508C40.376 36.7867 40.7848 37.5402 40.7427 38.2735C40.706 38.9775 40.1908 39.4725 39.5528 39.8373C39.1757 40.0647 38.7412 40.276 38.2863 40.4973C37.2045 41.0234 36.0076 41.6055 35.1932 42.5928C33.974 44.0045 32.4688 44.8772 31.0553 44.9872C29.6418 45.099 28.3182 44.45 27.6013 43.0237L27.5958 43.0182C27.5697 42.949 27.5454 42.89 27.523 42.8356C27.4653 42.6951 27.4199 42.5847 27.3868 42.4077C26.4008 42.4735 25.4823 42.3328 24.6448 42.2044C23.9204 42.0934 23.2567 41.9917 22.6623 42.041C21.6653 42.1039 20.8661 42.2696 20.1753 42.4127C19.5578 42.5407 19.027 42.6507 18.519 42.6533C18.0827 43.537 17.2687 44.175 16.3043 44.3877C14.9293 44.7543 13.206 44.3803 11.5083 43.526C10.4773 42.9711 9.26303 42.8279 8.13135 42.6944C7.52434 42.6228 6.9411 42.554 6.42267 42.426C5.68017 42.1858 5.01833 41.9365 4.69933 41.3242C4.38033 40.7027 4.43716 39.8502 4.89366 38.6035C5.02207 38.1946 4.94184 37.6503 4.84524 36.995C4.83702 36.9392 4.82868 36.8826 4.82033 36.8252C4.769 36.5758 4.7195 36.2073 4.7195 35.8425C4.71216 35.4612 4.7965 35.0853 4.9615 34.7388C5.32126 34.021 5.91228 33.7658 6.46354 33.5276C6.49097 33.5158 6.5183 33.504 6.5455 33.4922C6.68913 33.4309 6.82976 33.3772 6.96561 33.3253C7.37077 33.1706 7.73344 33.0321 8.00667 32.7588C8.17742 32.5672 8.34012 32.343 8.50901 32.1103C8.72634 31.8108 8.95394 31.4971 9.22217 31.2207C9.27154 31.1251 9.34021 31.0408 9.42383 30.9732C9.19833 29.4973 9.44033 27.9353 9.95 26.41C11.0298 23.1632 13.3068 20.0483 14.9293 18.1215C16.3043 16.1653 16.715 14.5868 16.8543 12.5848C16.8738 12.1381 16.8352 11.5454 16.7908 10.8636C16.5636 7.37332 16.1844 1.54924 22.666 1.0385ZM24.5397 7.2425H24.5158C24.1226 7.24289 23.7402 7.3698 23.4268 7.60733C23.065 7.83891 22.7883 8.18188 22.6385 8.5845C22.4431 9.02748 22.3491 9.50854 22.3635 9.9925V10.0292C22.367 10.1519 22.378 10.2744 22.3965 10.3958C22.582 10.4694 22.8174 10.5436 23.0524 10.6176C23.2099 10.6673 23.3673 10.7169 23.5093 10.7662C23.4873 10.6048 23.4763 10.4398 23.4727 10.2767V10.2418C23.4845 9.98963 23.5403 9.74144 23.6377 9.5085C23.7102 9.27639 23.8394 9.06596 24.0135 8.89617C24.1712 8.70917 24.3233 8.65417 24.4938 8.65417H24.5232C24.7047 8.65417 24.8557 8.77281 25.0081 8.89261L25.0127 8.89617C25.0488 8.95476 25.0822 9.00626 25.1132 9.0541C25.2112 9.20535 25.2855 9.32003 25.3482 9.50667C25.4263 9.7431 25.4641 9.99101 25.46 10.24V10.2767C25.449 10.5297 25.3958 10.779 25.2987 11.0137C25.2573 11.0863 25.1757 11.1644 25.0976 11.2393C25.0476 11.2872 24.9989 11.3337 24.9632 11.3767C25.1009 11.4051 25.2359 11.4456 25.3665 11.4977C25.4774 11.5867 25.5956 11.6303 25.7223 11.6771C25.7757 11.6968 25.8306 11.717 25.8872 11.7415C25.9426 11.7634 25.9966 11.7892 26.0485 11.8185C26.2203 11.6435 26.3534 11.4342 26.439 11.2043C26.6168 10.7958 26.7103 10.3555 26.714 9.91L26.7213 9.954C26.7222 9.96709 26.7246 9.98002 26.7287 9.9925V9.8C26.7287 9.81833 26.7259 9.83667 26.7232 9.855C26.7204 9.87333 26.7177 9.89167 26.7177 9.91C26.703 9.42417 26.6058 9.0575 26.4133 8.58267C26.2153 8.21417 25.9587 7.853 25.6103 7.6055C25.2657 7.35617 24.9302 7.2425 24.5397 7.2425ZM19.1515 7.35067H19.0855C18.7958 7.35067 18.5538 7.4735 18.3192 7.71733C18.0782 8.00351 17.9013 8.33795 17.8003 8.69817C17.6731 9.09315 17.6233 9.50898 17.6537 9.92283C17.668 10.345 17.7645 10.7604 17.9378 11.1457C18.0826 11.4065 18.2514 11.6532 18.442 11.8827C18.5311 11.8615 18.6136 11.8187 18.6822 11.758C18.748 11.6843 18.8058 11.6467 18.8698 11.6049C18.9109 11.578 18.9547 11.5495 19.0048 11.5087C18.9792 11.4977 18.9608 11.4812 18.9425 11.4647C18.8178 11.34 18.7225 11.219 18.6382 10.9733C18.6295 10.9464 18.6213 10.9211 18.6135 10.8969C18.5496 10.7 18.5111 10.5814 18.4915 10.361V10.3335C18.4698 10.1309 18.4841 9.92611 18.5337 9.7285C18.5607 9.54657 18.6367 9.37544 18.7537 9.2335C18.8563 9.06483 18.9645 9.0025 19.0947 8.9915H19.1332C19.1941 8.98752 19.2552 8.99587 19.3129 9.01602C19.3705 9.03618 19.4235 9.06775 19.4687 9.10883C19.6109 9.24277 19.7158 9.41148 19.773 9.59833C19.7858 9.63881 19.7979 9.67596 19.8092 9.71087C19.8662 9.88628 19.905 10.0055 19.9233 10.2088V10.2363C19.9417 10.3958 19.9398 10.5608 19.9178 10.7258C19.9701 10.6973 20.0226 10.6672 20.0762 10.6365C20.2463 10.5391 20.4264 10.4359 20.6383 10.3592C20.6632 10.3536 20.6875 10.3458 20.7126 10.3378C20.7375 10.3298 20.7631 10.3216 20.7905 10.3152V10.1685C20.8052 10.0475 20.807 9.9265 20.7942 9.68083V9.6735C20.7703 9.184 20.6787 8.8155 20.5137 8.45067C20.367 8.12617 20.1507 7.83467 19.883 7.59817C19.6465 7.43867 19.4118 7.35067 19.1515 7.35067ZM23.8265 11.2098C22.941 10.5975 22.2058 10.4783 21.5972 10.4783H21.599C21.2837 10.4783 20.9995 10.5315 20.7447 10.5975C20.0058 10.8377 19.5163 11.329 19.2065 11.6993V11.7048H19.2028C19.1423 11.758 19.0708 11.8093 18.8802 11.945C18.8088 11.99 18.7236 12.0537 18.623 12.1287C18.4574 12.2523 18.2501 12.407 17.9947 12.561C17.6317 12.7993 17.5125 13.1697 17.6408 13.661C17.7692 14.024 18.1652 14.5117 18.8985 14.9627H18.9077V14.9718C19.1865 15.1208 19.4099 15.3135 19.6231 15.4973C19.7589 15.6145 19.8905 15.728 20.0297 15.8243C20.2199 15.9557 20.4169 16.0769 20.62 16.1873C20.8828 16.2829 21.1621 16.3246 21.4413 16.3102C22.2045 16.3552 22.7698 16.0785 23.269 15.8342L23.2967 15.8207C23.5576 15.695 23.7982 15.5366 24.0381 15.3788C24.2601 15.2327 24.4814 15.0871 24.7175 14.9682V14.9645C25.7203 14.5997 26.4408 13.9892 26.6663 13.4208C26.7229 13.2834 26.7496 13.1354 26.7445 12.9868C26.7394 12.8383 26.7028 12.6925 26.637 12.5592V12.7993C26.5032 12.5555 26.2282 12.3172 25.7607 12.0678H25.7552C25.6086 12.0127 25.4746 11.963 25.3509 11.9171C24.6553 11.6593 24.2825 11.5211 23.8265 11.2098ZM29.9113 22.6058C28.9232 20.7762 27.3868 17.0032 26.7305 14.4053H26.7287C26.3455 14.9517 25.6708 15.4412 24.8147 15.6887H24.8055C24.5824 15.7556 24.3538 15.8976 24.104 16.0528C23.9021 16.1782 23.6865 16.3122 23.4488 16.422C22.9593 16.6677 22.3763 16.9133 21.621 16.9133C21.5616 16.9133 21.5005 16.9086 21.435 16.9035L21.4193 16.9023C20.6878 16.9023 20.1983 16.6567 19.8133 16.2882C19.626 16.1704 19.4595 16.0231 19.2918 15.8748C19.115 15.7184 18.9368 15.5608 18.7317 15.4357C18.6433 15.3534 18.4753 15.2449 18.3474 15.1624C18.2843 15.1216 18.2309 15.0872 18.2018 15.0653C18.0598 17.7587 16.5369 21.0396 15.4344 23.4147C15.2131 23.8914 15.0087 24.3316 14.8358 24.7233C14.1222 26.3938 13.7202 28.1808 13.6497 29.996C11.7833 27.5082 13.1473 24.3072 13.8733 22.9303C14.6892 21.3922 14.8175 21.0255 14.6213 21.1447C13.8862 22.3418 12.7348 24.2485 12.2857 26.2102C12.051 27.2368 12.0107 28.247 12.3132 29.226C12.6138 30.1903 13.2665 31.052 14.4215 31.7853C15.9927 32.7497 17.067 33.7177 17.6995 34.5738C18.332 35.4282 18.53 36.2165 18.332 36.706C18.2262 36.9767 18.02 37.196 17.7563 37.3183C17.5235 37.3862 17.2375 37.4412 16.9038 37.4412C17.1014 37.6184 17.2714 37.8241 17.408 38.0517C17.694 38.345 17.9416 38.6733 18.145 39.0288C21.3863 41.2362 25.1997 40.3818 27.9497 38.4165C27.9605 38.3792 27.9714 38.3419 27.9822 38.3047C28.2443 37.4049 28.4993 36.5295 28.5345 35.9415V35.936C28.5987 34.6288 28.672 33.4867 28.9103 32.5095C29.1505 31.5598 29.5758 30.8247 30.3642 30.3333C30.4616 30.318 30.56 30.2827 30.6563 30.2481C30.6869 30.2371 30.7173 30.2262 30.7473 30.216C30.7558 30.0996 30.7642 30.0546 30.7769 29.9868C30.778 29.9809 30.7792 29.9748 30.7803 29.9685C31.0077 28.5 32.3478 28.3827 34.0253 29.116C35.6442 29.8475 36.2547 30.5038 35.9778 31.36H36.0072C36.0438 31.36 36.0805 31.3594 36.1172 31.3588C36.1905 31.3576 36.2638 31.3563 36.3372 31.36V31.4388C36.6727 30.3407 35.9742 29.6 34.0822 28.621C33.9694 28.6097 33.8595 28.5631 33.7512 28.5171C33.7362 28.5108 33.7213 28.5044 33.7063 28.4982C33.9612 27.4605 33.9062 26.4173 33.6678 25.4365C33.1527 23.264 31.7318 21.361 30.6502 20.382C30.4503 20.382 30.4723 20.6295 30.8757 20.9962C31.8712 21.911 34.0565 25.2037 32.8722 28.2672C32.5367 28.1828 32.2177 28.1407 31.9317 28.1498C31.4788 25.6473 30.4357 23.5848 29.9113 22.6058ZM32.3533 33.3528C31.3652 33.1108 30.8042 31.9283 30.7363 30.8283V30.832L30.6538 30.8723C30.0433 31.2152 29.6987 31.8238 29.4787 32.6763C29.2605 33.5545 29.189 34.693 29.1212 36.002C29.0756 36.7223 28.8292 37.5563 28.5746 38.4181C28.4709 38.7694 28.3658 39.1252 28.2723 39.4798C27.9497 40.699 27.7847 41.8448 28.1697 42.5782L28.1788 42.5837C28.7967 43.8633 29.8472 44.3272 31.0645 44.23C32.2855 44.1365 33.6642 43.3977 34.7972 42.0465C35.7047 40.9513 36.9722 40.3422 38.0484 39.8251C38.5178 39.5996 38.9508 39.3915 39.3017 39.1682C39.8755 38.8033 40.1872 38.5595 40.2147 38.0792L40.2202 38.0993V38.0663L40.2147 38.0792C40.2312 37.6502 39.9397 36.9553 39.0487 35.9892C38.6065 35.4932 38.4694 34.8431 38.324 34.1536C38.3156 34.1142 38.3073 34.0745 38.2988 34.0348C38.1412 33.4005 37.9633 32.7937 37.582 32.4288L37.5765 32.4233C37.0815 31.9503 36.6012 31.9192 36.0237 31.932L35.6368 31.9503C34.9823 32.7423 33.4313 33.5985 32.3533 33.3528ZM10.929 30.9182H10.9107C10.3918 30.9182 10.0435 31.1657 9.69517 31.5342C9.5004 31.7412 9.31978 31.9881 9.13328 32.2431C8.92831 32.5233 8.71624 32.8132 8.4705 33.0705V33.0742H8.46683C8.1027 33.429 7.67066 33.5917 7.25569 33.7479C7.0961 33.808 6.93904 33.8671 6.78933 33.9358C6.72367 33.9655 6.65958 33.9934 6.59713 34.0206C6.14698 34.2164 5.78193 34.3753 5.52433 34.913C5.29333 35.3347 5.34467 35.9012 5.4455 36.6363C5.45614 36.7047 5.46735 36.774 5.47868 36.844C5.57861 37.4618 5.68765 38.136 5.495 38.686L5.49133 38.6915V38.6988C5.05867 39.8502 5.06417 40.5872 5.26583 40.9538C5.473 41.3205 5.91483 41.5662 6.59133 41.6872C6.96665 41.7547 7.39979 41.7941 7.86764 41.8367C9.08866 41.9478 10.5461 42.0804 11.8292 42.7908C13.4333 43.6452 15.0375 43.889 16.22 43.6415C17.4025 43.3738 18.178 42.6332 18.2825 41.2692V41.2582C18.3577 40.4533 17.7417 39.4762 16.946 38.3798C16.7146 38.075 16.4714 37.7766 16.2319 37.4828C15.6462 36.764 15.0824 36.0723 14.7662 35.3787L14.7607 35.3732L13.0923 32.3225C12.5112 31.5543 11.9062 31.0447 11.2168 30.9438C11.1215 30.9273 11.0262 30.9182 10.929 30.9182ZM23.5083 14.5427C23.6897 14.4504 23.8657 14.3609 24.0355 14.2825L24.0337 14.2843C24.4698 14.0202 24.892 13.7339 25.2987 13.4263C25.6305 13.0633 25.8028 12.847 25.5682 12.8177C25.4293 12.8177 25.3757 12.8992 25.2985 13.0166C25.2433 13.1007 25.176 13.2031 25.0567 13.3072C24.8433 13.4469 24.6017 13.6265 24.3669 13.8009C24.1922 13.9307 24.0212 14.0577 23.8687 14.1633C23.2362 14.5282 22.1948 15.0177 21.3057 15.0177C20.4161 15.0177 19.7068 14.5318 19.1721 14.1656L19.1662 14.1615C19.0678 14.0713 18.976 13.9809 18.8905 13.8966C18.7447 13.753 18.6171 13.6272 18.5062 13.551C18.4612 13.5086 18.4274 13.4544 18.394 13.4007C18.332 13.3013 18.2711 13.2034 18.1413 13.1843C18.013 13.1843 17.9763 13.5492 18.277 13.7948C18.3752 13.8656 18.4936 13.9772 18.6302 14.106C18.7307 14.2008 18.8412 14.3049 18.9608 14.409C19.5292 14.7738 20.3138 15.2633 21.3075 15.2633C22.0911 15.2633 22.8372 14.8839 23.5083 14.5427ZM22.0518 11.34V11.3803C22.0735 11.4531 22.1637 11.4663 22.2486 11.4788C22.2856 11.4842 22.3217 11.4895 22.3507 11.4995C22.3795 11.5142 22.4061 11.5324 22.4323 11.5502C22.4875 11.5879 22.5407 11.6242 22.6092 11.6242C22.7008 11.6242 22.8457 11.593 22.8567 11.5032C22.875 11.3785 22.6972 11.2557 22.5817 11.2557C22.435 11.1988 22.2407 11.1695 22.105 11.2483C22.0757 11.263 22.0408 11.3033 22.0518 11.34ZM20.8356 11.482C20.9223 11.4694 21.0157 11.4557 21.0417 11.3803V11.3767H21.0362V11.34C21.0472 11.3033 21.016 11.2648 20.9812 11.2483C20.8437 11.1713 20.653 11.2007 20.5063 11.2575C20.3927 11.2575 20.213 11.3803 20.2313 11.5013C20.2423 11.5893 20.389 11.626 20.4807 11.626C20.5494 11.626 20.6048 11.588 20.6612 11.5495C20.686 11.5325 20.711 11.5154 20.7373 11.5013C20.7656 11.4923 20.8 11.4872 20.8356 11.482Z"
                fill="white"
              />
            </svg>
            <svg
              class="light-icon"
              role="img"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"
              />
            </svg>
            Linux
          </div>
          <div class="description apple">
            <a ref="downloadLinux" :href="linuxLinks.appImage" download="">
              <DownloadIcon />
              <span> Download the AppImage </span>
            </a>
            <a :href="linuxLinks.deb" download="">
              <DownloadIcon />
              <span> Download the Deb </span>
            </a>
            <a :href="linuxLinks.thirdParty" download="">
              <LinkIcon />
              <span> Third-party packages </span>
            </a>
          </div>
        </div>
      </div>
      <p class="terms">
        By downloading the Modrinth App you agree to our
        <nuxt-link to="/legal/terms"> Terms</nuxt-link> and
        <nuxt-link to="/legal/privacy">Privacy Policy.</nuxt-link>
      </p>
    </div>
    <div class="logo-banner">
      <svg
        v-if="$theme.active === 'light'"
        viewBox="0 0 865 512"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        class="light-height"
      >
        <g clip-path="url(#clip0_419_237)">
          <rect x="176" width="512" height="512" fill="url(#paint0_linear_419_237)" />
          <g opacity="0.5">
            <path
              d="M176 274.069H219.65C221.89 299.001 228.25 322.42 238.87 345.249C245.4 342.033 251.37 338.016 257.52 334.3C263.76 330.534 270.07 326.908 276.62 323.061C276.21 321.669 275.95 320.216 275.37 318.914C271.05 309.378 268.47 299.251 266.32 289.114C265.03 283.054 264.41 276.743 263.73 270.513C262.89 262.86 262.94 255.257 263.11 247.654C263.26 241.083 264.11 234.482 265.21 227.991C266.88 218.064 269.39 208.308 272.94 198.852C276.79 188.594 281.19 178.588 287.21 169.382C291.19 163.292 294.97 157.021 299.53 151.382C306.3 142.998 313.75 135.214 322.12 128.273C330.05 121.692 338.16 115.421 347.18 110.483C354.15 106.676 361.39 103.341 368.66 100.115C379.13 95.4674 390.24 92.873 401.39 90.4289C410.67 88.4055 430.03 87.3237 438.96 88.6158C438.51 95.9282 436.63 103.05 435.44 110.262C434.26 117.394 432.94 124.496 431.6 132.109C428.15 132.44 424.73 132.7 421.32 133.121C416.25 133.752 411.15 134.263 406.15 135.265C395.4 137.428 385.13 141.044 375.33 145.973C366.85 150.23 358.98 155.398 351.89 161.709C348.8 164.464 345.52 167.048 342.67 170.033C335.77 177.225 329.99 185.279 324.68 193.713C320.48 200.364 317.45 207.536 314.8 214.839C312.05 222.422 310.33 230.315 308.95 238.308C307.43 247.093 307.9 255.898 308.2 264.653C308.48 272.867 310.18 281 312.49 288.934C313.45 292.239 314.44 295.535 315.62 299.481C332.06 289.705 348.08 280.169 364.47 270.422C361.92 263.631 359.46 257.16 357.05 250.669C354.63 244.178 352.26 237.667 349.78 230.926C352.79 227.871 355.71 224.936 358.6 221.971C369.97 210.291 381.46 198.712 392.6 186.821C395.77 183.436 399.36 181.913 403.66 181.062C414.16 178.978 424.63 176.734 435.11 174.541C441.87 173.128 448.62 171.686 455.69 170.193C461.36 177.175 467.08 184.217 472.94 191.439C471.8 192.671 470.85 193.783 469.82 194.815C463 201.656 455.97 208.308 449.41 215.4C446.18 218.896 442.34 220.709 438.01 221.961C433.09 223.383 428.16 224.745 423.25 226.198C422.5 226.418 421.73 226.909 421.19 227.48C416.62 232.298 412.15 237.216 407.55 241.995C405.9 243.697 405.89 245.31 406.67 247.424C408.91 253.474 410.92 259.604 413.12 265.674C413.5 266.716 414.27 267.668 415.04 268.499C419.55 273.377 424.05 278.266 428.73 282.974C429.48 283.725 431.24 284.055 432.33 283.775C438.57 282.182 444.72 280.289 450.94 278.636C453.49 277.955 455.32 276.443 457.01 274.559C460.1 271.094 463.55 267.898 466.27 264.172C469.55 259.684 473.91 257.901 479.09 256.82C483.44 255.908 487.58 254.045 491.83 252.673C496.23 251.25 500.69 249.998 505.07 248.495C507.08 247.804 508.18 248.305 508.99 250.198C512.16 257.671 515.4 265.114 518.76 272.917C515.97 276.342 513.04 279.918 510.13 283.515C505.45 289.304 500.81 295.124 496.12 300.904C492.16 305.782 487.86 310.42 484.26 315.549C481.66 319.255 478.09 320.717 474.16 321.959C462.33 325.716 450.52 329.502 438.69 333.268C431.61 335.522 424.51 337.756 416.91 340.16C415.33 338.648 413.45 337.055 411.81 335.232C407.16 330.053 402.62 324.794 398.03 319.565C395.63 316.831 393.3 314.006 390.79 311.382C388.08 308.557 387.52 308.557 384.41 310.42C377.88 314.337 371.33 318.243 364.82 322.19C356.32 327.328 347.85 332.507 339.09 337.836C341.26 341.482 344.39 344.257 347.49 346.811C353.22 351.539 359.26 355.907 365.37 360.144C376.53 367.867 388.92 372.815 402.07 376.081C409.82 378.004 417.59 379.286 425.6 379.076C426.74 379.046 427.88 379.446 429.02 379.516C430.64 379.617 432.27 379.667 433.88 379.587C435.19 379.526 436.48 379.106 437.79 379.076C442.03 378.966 442.07 378.996 443.29 383.443C446.74 396.014 450.16 408.585 453.58 421.167C453.7 421.597 453.65 422.068 453.7 422.88C449 423.28 444.42 423.781 439.83 424.022C436.17 424.212 432.5 423.961 428.83 424.092C414.81 424.593 401.08 422.469 387.69 418.733C366.28 412.763 346.56 403.216 328.88 389.523C320.7 383.183 313.41 375.95 306.16 368.648C304.89 367.366 303.97 365.743 302.78 364.371C300.52 361.746 299.87 361.526 296.85 363.189C292.49 365.583 288.24 368.167 283.98 370.722C277.72 374.458 271.49 378.234 265.25 382.001C264.56 382.421 263.91 382.902 263.18 383.403C263.7 387.54 267.09 389.654 269.19 392.448C272.55 396.946 276.45 401.113 280.58 404.939C286.77 410.669 293.38 415.938 299.77 421.457C306.94 427.658 314.86 432.756 322.98 437.604C332.25 443.144 341.84 448.092 351.94 451.808C361.7 455.394 371.65 458.54 381.69 461.244C388.49 463.077 395.62 463.678 402.56 465.061C414.21 467.385 425.99 467.224 437.75 466.924C444.49 466.754 451.21 465.622 457.94 465.071C465.69 464.44 473.25 462.767 480.79 461.014C492.41 458.319 503.64 454.453 514.64 449.795C530.77 442.963 545.63 433.968 559.65 423.601C570.63 415.477 580.59 406.061 589.5 395.774C598.25 385.667 606.72 375.209 612.97 363.219C615.56 358.241 618.38 353.382 621.25 348.184C635.52 353.322 649.65 358.411 664.03 363.59C663.75 364.691 663.65 365.663 663.26 366.484C655.92 381.77 647.35 396.285 636.95 409.727C628.93 420.105 620.71 430.292 611.17 439.307C604.8 445.327 598.41 451.358 591.25 456.496C584.53 461.314 578.16 466.653 571.24 471.151C556.34 480.857 540.49 488.721 523.9 495.262C508.29 501.412 492.07 505.198 475.79 508.774C468.91 510.287 461.7 510.297 454.64 511.058C453.74 511.158 452.89 511.679 452.02 512H409.02C405.58 510.147 401.69 510.908 398.04 510.127C392.73 508.995 387.16 509.055 381.91 507.763C370.54 504.958 359.16 502.043 348.04 498.387C335.77 494.35 323.98 489.101 312.56 482.941C300.86 476.63 289.55 469.779 278.84 461.945C269.2 454.894 260.11 447.171 251.49 438.927C248.39 435.952 245.88 432.356 242.77 429.391C235.89 422.83 230.17 415.267 224.57 407.704C218.48 399.48 212.95 390.755 208.03 381.78C197.57 362.698 188.92 342.835 183.67 321.579C180.7 309.558 177.71 297.578 176.89 285.177C176.86 284.787 176.32 284.436 176.02 284.065C176.02 280.73 176.02 277.404 176.02 274.069H176Z"
              fill="#05CE45"
            />
            <path
              d="M688 277.074C687.69 277.615 687.11 278.136 687.09 278.686C686.56 293.612 683.03 308.016 679.73 322.45C675.99 322.05 642.32 309.939 636.41 306.894C637.6 300.173 638.85 293.431 639.97 286.66C641.13 279.648 641.76 272.606 642.1 265.484C642.43 258.563 642.1 251.691 642.02 244.789C641.94 237.858 641.47 231.006 639.48 223.784C625.94 227.39 612.76 230.896 599.27 234.492C599.19 235.964 598.74 237.507 599.09 238.839C600.63 244.749 599.88 250.739 599.99 256.699C600.43 280.079 595.08 302.226 585.83 323.592C580.73 335.372 574.24 346.311 566.66 356.598C559.95 365.713 552.43 374.228 543.81 381.53C534.36 389.543 524.76 397.497 513.41 402.896C508.18 405.39 503.03 408.045 497.83 410.599C497.43 410.799 496.9 410.759 496.49 410.819C494.33 407.514 486.2 378.274 484.59 368.177C533.62 341.532 557.58 300.664 554.95 244.639C548.26 194.314 521.33 159.555 474.14 139.752C476.69 125.278 479.28 110.593 481.96 95.3973C486 96.7796 489.75 97.8414 493.32 99.3439C504.19 103.932 514.89 108.93 524.68 115.541C537.67 124.316 549.44 134.653 559.65 146.473C570.52 159.055 579.39 173.078 585.96 188.434C587.1 191.099 587.3 191.279 589.95 190.578C602.53 187.272 615.11 183.917 628.08 180.481C627.53 178.608 627.18 176.845 626.5 175.232C618.95 157.262 609.28 140.493 597.33 125.107C590.3 116.052 582.71 107.398 574.08 99.8448C561.23 88.6058 547.51 78.599 532.54 70.1948C521.76 64.1446 510.5 59.3665 498.93 55.3798C488.2 51.6836 477.17 48.8788 465.88 46.9756C450.75 44.4313 435.58 43.7201 420.33 44.221C414.75 44.4013 409.21 45.6033 403.63 46.004C394.2 46.6851 385.07 48.979 375.97 51.2128C367.14 53.3764 358.57 56.4516 350.19 60.0276C346.54 61.5903 342.73 62.8324 339.19 64.6053C333.26 67.5804 327.36 70.6455 321.67 74.0713C316.29 77.3068 311.08 80.8628 306.03 84.5991C300.85 88.4255 295.84 92.5124 290.96 96.7195C286.31 100.726 281.67 104.793 277.47 109.261C267.13 120.259 257.6 131.919 250.05 145.061C246.17 151.812 242.21 158.564 238.88 165.596C231.64 180.881 226.13 196.788 222.94 213.466C222.13 217.674 220.91 221.79 220.92 226.148C220.92 227.36 220.13 228.582 219.6 230.084H176C176 229.083 176 228.081 176 227.089C176.32 226.548 176.92 226.008 176.9 225.477C176.77 220.298 178.02 215.33 178.92 210.291C181.03 198.581 183.94 187.072 187.69 175.803C189.88 169.212 192.51 162.771 195.06 156.31C199.14 145.933 204.4 136.146 210.05 126.57C215.44 117.444 221.43 108.74 227.82 100.265C233.72 92.4423 240.04 84.9797 246.83 77.9879C253.05 71.5771 260.16 66.0378 266.65 59.8774C274.64 52.3046 283.78 46.2644 292.92 40.3544C303.06 33.8034 313.58 27.8033 324.82 23.0953C335.82 18.4876 346.77 13.8598 358.24 10.4941C373.31 6.07663 388.68 3.1517 404.23 1.2585C411.77 0.336948 419.45 0.136611 427.06 0.206729C436.84 0.296881 446.65 -0.714826 456.41 1.05816C463.53 2.35034 470.77 2.98141 477.92 4.14337C489.5 6.02654 500.81 9.12177 511.97 12.6778C521.29 15.6428 530.2 19.6796 539.17 23.5661C548.81 27.7432 557.74 33.1222 566.71 38.4312C572.54 41.877 577.86 46.1642 583.55 49.8705C592.62 55.7704 600.49 63.1529 608.47 70.325C615 76.1949 620.72 82.9663 626.72 89.4172C628.86 91.7211 630.91 94.1352 632.73 96.6794C638.01 104.082 643.32 111.474 648.31 119.057C655.9 130.577 662.42 142.737 667.58 155.519C673.71 170.684 679.37 186.06 682.52 202.197C683.75 208.498 685.02 214.799 685.94 221.149C686.6 225.717 686.7 230.365 687.12 234.973C687.16 235.373 687.69 235.724 687.99 236.095V277.084L688 277.074Z"
              fill="#05CE45"
            />
          </g>
        </g>
        <rect y="194" width="865" height="318" fill="url(#paint1_linear_419_237)" />
        <defs>
          <linearGradient
            id="paint0_linear_419_237"
            x1="432"
            y1="0"
            x2="432"
            y2="512"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="#E7FFEF" />
            <stop offset="0.678759" stop-color="white" />
          </linearGradient>
          <linearGradient
            id="paint1_linear_419_237"
            x1="432.5"
            y1="194"
            x2="432.5"
            y2="315.861"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="#F9FFFB" stop-opacity="0" />
            <stop offset="1" stop-color="#F9FFFB" />
          </linearGradient>
          <clipPath id="clip0_419_237">
            <rect x="176" width="512" height="512" rx="256" fill="white" />
          </clipPath>
        </defs>
      </svg>
      <svg v-else viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_127_331)">
          <rect width="512" height="512" fill="url(#paint0_linear_127_331)" />
          <g style="mix-blend-mode: overlay">
            <g opacity="0.5">
              <path
                d="M0 274.069H43.65C45.89 299.001 52.25 322.42 62.87 345.249C69.4 342.033 75.37 338.016 81.52 334.3C87.76 330.534 94.07 326.908 100.62 323.061C100.21 321.669 99.95 320.216 99.37 318.914C95.05 309.378 92.47 299.251 90.32 289.114C89.03 283.054 88.41 276.743 87.73 270.513C86.89 262.86 86.94 255.257 87.11 247.654C87.26 241.083 88.11 234.482 89.21 227.991C90.88 218.064 93.39 208.308 96.94 198.852C100.79 188.594 105.19 178.588 111.21 169.382C115.19 163.292 118.97 157.021 123.53 151.382C130.3 142.998 137.75 135.214 146.12 128.273C154.05 121.692 162.16 115.421 171.18 110.483C178.15 106.676 185.39 103.341 192.66 100.115C203.13 95.4674 214.24 92.873 225.39 90.4289C234.67 88.4055 254.03 87.3237 262.96 88.6158C262.51 95.9282 260.63 103.05 259.44 110.262C258.26 117.394 256.94 124.496 255.6 132.109C252.15 132.44 248.73 132.7 245.32 133.121C240.25 133.752 235.15 134.263 230.15 135.265C219.4 137.428 209.13 141.044 199.33 145.973C190.85 150.23 182.98 155.398 175.89 161.709C172.8 164.464 169.52 167.048 166.67 170.033C159.77 177.225 153.99 185.279 148.68 193.713C144.48 200.364 141.45 207.536 138.8 214.839C136.05 222.422 134.33 230.315 132.95 238.308C131.43 247.093 131.9 255.898 132.2 264.653C132.48 272.867 134.18 281 136.49 288.934C137.45 292.239 138.44 295.535 139.62 299.481C156.06 289.705 172.08 280.169 188.47 270.422C185.92 263.631 183.46 257.16 181.05 250.669C178.63 244.178 176.26 237.667 173.78 230.926C176.79 227.871 179.71 224.936 182.6 221.971C193.97 210.291 205.46 198.712 216.6 186.821C219.77 183.436 223.36 181.913 227.66 181.062C238.16 178.978 248.63 176.734 259.11 174.541C265.87 173.128 272.62 171.686 279.69 170.193C285.36 177.175 291.08 184.217 296.94 191.439C295.8 192.671 294.85 193.783 293.82 194.815C287 201.656 279.97 208.308 273.41 215.4C270.18 218.896 266.34 220.709 262.01 221.961C257.09 223.383 252.16 224.745 247.25 226.198C246.5 226.418 245.73 226.909 245.19 227.48C240.62 232.298 236.15 237.216 231.55 241.995C229.9 243.697 229.89 245.31 230.67 247.424C232.91 253.474 234.92 259.604 237.12 265.674C237.5 266.716 238.27 267.668 239.04 268.499C243.55 273.377 248.05 278.266 252.73 282.974C253.48 283.725 255.24 284.055 256.33 283.775C262.57 282.182 268.72 280.289 274.94 278.636C277.49 277.955 279.32 276.443 281.01 274.559C284.1 271.094 287.55 267.898 290.27 264.172C293.55 259.684 297.91 257.901 303.09 256.82C307.44 255.908 311.58 254.045 315.83 252.673C320.23 251.25 324.69 249.998 329.07 248.495C331.08 247.804 332.18 248.305 332.99 250.198C336.16 257.671 339.4 265.114 342.76 272.917C339.97 276.342 337.04 279.919 334.13 283.515C329.45 289.304 324.81 295.124 320.12 300.904C316.16 305.782 311.86 310.42 308.26 315.549C305.66 319.255 302.09 320.717 298.16 321.959C286.33 325.716 274.52 329.502 262.69 333.268C255.61 335.522 248.51 337.756 240.91 340.16C239.33 338.648 237.45 337.055 235.81 335.232C231.16 330.053 226.62 324.794 222.03 319.565C219.63 316.831 217.3 314.006 214.79 311.382C212.08 308.557 211.52 308.557 208.41 310.42C201.88 314.337 195.33 318.243 188.82 322.19C180.32 327.328 171.85 332.507 163.09 337.836C165.26 341.482 168.39 344.257 171.49 346.811C177.22 351.539 183.26 355.907 189.37 360.144C200.53 367.867 212.92 372.815 226.07 376.081C233.82 378.004 241.59 379.286 249.6 379.076C250.74 379.046 251.88 379.446 253.02 379.516C254.64 379.617 256.27 379.667 257.88 379.587C259.19 379.526 260.48 379.106 261.79 379.076C266.03 378.966 266.07 378.996 267.29 383.443C270.74 396.014 274.16 408.585 277.58 421.167C277.7 421.597 277.65 422.068 277.7 422.88C273 423.28 268.42 423.781 263.83 424.022C260.17 424.212 256.5 423.961 252.83 424.092C238.81 424.592 225.08 422.469 211.69 418.733C190.28 412.763 170.56 403.216 152.88 389.523C144.7 383.183 137.41 375.95 130.16 368.648C128.89 367.366 127.97 365.743 126.78 364.371C124.52 361.746 123.87 361.526 120.85 363.189C116.49 365.583 112.24 368.167 107.98 370.722C101.72 374.458 95.49 378.234 89.25 382.001C88.56 382.421 87.91 382.902 87.18 383.403C87.7 387.54 91.09 389.654 93.19 392.448C96.55 396.946 100.45 401.113 104.58 404.939C110.77 410.669 117.38 415.938 123.77 421.457C130.94 427.658 138.86 432.756 146.98 437.604C156.25 443.144 165.84 448.092 175.94 451.808C185.7 455.394 195.65 458.54 205.69 461.244C212.49 463.077 219.62 463.678 226.56 465.061C238.21 467.385 249.99 467.224 261.75 466.924C268.49 466.754 275.21 465.622 281.94 465.071C289.69 464.44 297.25 462.767 304.79 461.014C316.41 458.319 327.64 454.453 338.64 449.795C354.77 442.964 369.63 433.968 383.65 423.601C394.63 415.477 404.59 406.061 413.5 395.774C422.25 385.667 430.72 375.209 436.97 363.219C439.56 358.241 442.38 353.382 445.25 348.184C459.52 353.322 473.65 358.411 488.03 363.59C487.75 364.691 487.65 365.663 487.26 366.484C479.92 381.77 471.35 396.285 460.95 409.727C452.93 420.105 444.71 430.292 435.17 439.307C428.8 445.327 422.41 451.358 415.25 456.496C408.53 461.314 402.16 466.653 395.24 471.151C380.34 480.857 364.49 488.721 347.9 495.262C332.29 501.412 316.07 505.198 299.79 508.774C292.91 510.287 285.7 510.297 278.64 511.058C277.74 511.158 276.89 511.679 276.02 512H233.02C229.58 510.147 225.69 510.908 222.04 510.127C216.73 508.995 211.16 509.055 205.91 507.763C194.54 504.958 183.16 502.043 172.04 498.387C159.77 494.35 147.98 489.101 136.56 482.941C124.86 476.63 113.55 469.779 102.84 461.945C93.2 454.894 84.11 447.171 75.49 438.927C72.39 435.952 69.88 432.356 66.77 429.391C59.89 422.829 54.17 415.267 48.57 407.704C42.48 399.48 36.95 390.755 32.03 381.78C21.57 362.698 12.92 342.835 7.67 321.579C4.7 309.558 1.71 297.578 0.89 285.177C0.86 284.787 0.32 284.436 0.02 284.065C0.02 280.73 0.02 277.404 0.02 274.069H0Z"
                fill="white"
              />
              <path
                d="M512 277.074C511.69 277.615 511.11 278.136 511.09 278.686C510.56 293.612 507.03 308.016 503.73 322.45C499.99 322.05 466.32 309.939 460.41 306.894C461.6 300.173 462.85 293.431 463.97 286.66C465.13 279.648 465.76 272.606 466.1 265.484C466.43 258.563 466.1 251.691 466.02 244.789C465.94 237.858 465.47 231.006 463.48 223.784C449.94 227.39 436.76 230.896 423.27 234.492C423.19 235.964 422.74 237.507 423.09 238.839C424.63 244.749 423.88 250.739 423.99 256.699C424.43 280.079 419.08 302.226 409.83 323.592C404.73 335.372 398.24 346.311 390.66 356.598C383.95 365.713 376.43 374.228 367.81 381.53C358.36 389.543 348.76 397.497 337.41 402.896C332.18 405.39 327.03 408.045 321.83 410.599C321.43 410.799 320.9 410.759 320.49 410.819C318.33 407.514 310.2 378.274 308.59 368.177C357.62 341.532 381.58 300.664 378.95 244.639C372.26 194.314 345.33 159.555 298.14 139.752C300.69 125.278 303.28 110.593 305.96 95.3973C310 96.7796 313.75 97.8414 317.32 99.3439C328.19 103.932 338.89 108.93 348.68 115.541C361.67 124.316 373.44 134.654 383.65 146.473C394.52 159.055 403.39 173.078 409.96 188.434C411.1 191.099 411.3 191.279 413.95 190.578C426.53 187.272 439.11 183.917 452.08 180.481C451.53 178.608 451.18 176.845 450.5 175.232C442.95 157.262 433.28 140.493 421.33 125.107C414.3 116.052 406.71 107.398 398.08 99.8448C385.23 88.6058 371.51 78.599 356.54 70.1948C345.76 64.1446 334.5 59.3665 322.93 55.3798C312.2 51.6836 301.17 48.8788 289.88 46.9756C274.75 44.4313 259.58 43.7201 244.33 44.221C238.75 44.4013 233.21 45.6033 227.63 46.004C218.2 46.6851 209.07 48.979 199.97 51.2128C191.14 53.3764 182.57 56.4516 174.19 60.0276C170.54 61.5903 166.73 62.8324 163.19 64.6053C157.26 67.5804 151.36 70.6455 145.67 74.0713C140.29 77.3068 135.08 80.8628 130.03 84.5991C124.85 88.4255 119.84 92.5124 114.96 96.7195C110.31 100.726 105.67 104.793 101.47 109.261C91.13 120.259 81.6 131.919 74.05 145.061C70.17 151.812 66.21 158.564 62.88 165.596C55.64 180.881 50.13 196.788 46.94 213.466C46.13 217.674 44.91 221.79 44.92 226.148C44.92 227.36 44.13 228.582 43.6 230.084H0C0 229.083 0 228.081 0 227.089C0.32 226.548 0.92 226.008 0.9 225.477C0.77 220.298 2.02 215.33 2.92 210.291C5.03 198.581 7.94 187.072 11.69 175.803C13.88 169.212 16.51 162.771 19.06 156.31C23.14 145.933 28.4 136.146 34.05 126.57C39.44 117.444 45.43 108.74 51.82 100.265C57.72 92.4423 64.04 84.9797 70.83 77.9879C77.05 71.5771 84.16 66.0378 90.65 59.8774C98.64 52.3046 107.78 46.2644 116.92 40.3544C127.06 33.8034 137.58 27.8033 148.82 23.0953C159.82 18.4876 170.77 13.8598 182.24 10.4941C197.31 6.07663 212.68 3.1517 228.23 1.2585C235.77 0.336948 243.45 0.136611 251.06 0.206729C260.84 0.296881 270.65 -0.714826 280.41 1.05816C287.53 2.35034 294.77 2.98141 301.92 4.14337C313.5 6.02654 324.81 9.12177 335.97 12.6778C345.29 15.6428 354.2 19.6796 363.17 23.5661C372.81 27.7432 381.74 33.1222 390.71 38.4312C396.54 41.877 401.86 46.1642 407.55 49.8705C416.62 55.7705 424.49 63.1529 432.47 70.325C439 76.1949 444.72 82.9663 450.72 89.4172C452.86 91.7211 454.91 94.1352 456.73 96.6795C462.01 104.082 467.32 111.474 472.31 119.057C479.9 130.577 486.42 142.737 491.58 155.519C497.71 170.684 503.37 186.06 506.52 202.197C507.75 208.498 509.02 214.799 509.94 221.149C510.6 225.717 510.7 230.365 511.12 234.973C511.16 235.373 511.69 235.724 511.99 236.095V277.084L512 277.074Z"
                fill="white"
              />
            </g>
          </g>
        </g>
        <defs>
          <linearGradient
            id="paint0_linear_127_331"
            x1="256"
            y1="0"
            x2="256"
            y2="512"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="#05CE45" />
            <stop offset="0.678759" stop-color="#051F10" />
          </linearGradient>
          <clipPath id="clip0_127_331">
            <rect width="512" height="512" rx="256" fill="white" />
          </clipPath>
        </defs>
      </svg>
      <div class="overlay">
        <h2 class="main-header">
          Read more about <br />
          <strong class="main-header-strong">Modrinth</strong>
        </h2>
        <a
          href="https://blog.modrinth.com/?utm_source=website&utm_source=homepage&utm_campaign=newsletter"
          class="iconified-button brand-button"
        >
          Visit the blog
        </a>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.landing-hero {
  position: relative;
  background: #0f1121 url("https://cdn-raw.modrinth.com/app-landing/cube-black.png") no-repeat
    center 4rem;
  background-size: cover;
  padding: 6rem 1rem 12rem 1rem;
  margin-top: -4rem;
  display: flex;
  justify-content: center;
  align-items: center;
  text-align: center;
  flex-direction: column;
  isolation: isolate;

  .main-subheader {
    font-size: 1.625rem;
  }

  h2 {
    line-height: 125%;
    margin: 0 0 1.625rem;
    font-weight: 400;
    line-break: loose;
    color: var(--landing-color-subheading);
    max-width: 1096px;
    mask-image: none;
  }

  .button-group {
    width: fit-content;
    margin: 0 auto;
    justify-content: center;
    mask-image: none;

    .outline-button {
      color: var(--landing-color-heading);
      background: none;
      border: 1px var(--landing-color-heading) solid;
    }
  }

  img {
    width: 100%;
    max-width: 65rem;
    height: auto;
    mask-image: none;
    z-index: 1;
  }

  .bottom-transition {
    z-index: -1;
  }
}

.main-header {
  font-size: 5.25rem;
  font-weight: 600;
  line-height: 100%;
  margin: 2rem 0;
}

.subheader {
  font-size: 3.5rem;
  font-weight: 600;
  line-height: 100%;
  margin: 0 auto;
  padding: 0 4rem 4rem;
}

.features {
  position: relative;
  width: 100%;
  background: var(--landing-transition-gradient-end);
  align-content: center;
  justify-content: center;
  display: flex;
  flex-direction: column;

  h3 {
    font-weight: 500;
    font-size: var(--font-size-xl) !important;
  }

  p {
    font-size: var(--font-size-md) !important;
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    grid-template-rows: repeat(4, 1fr);
    max-width: min(100%, 1096px);
    width: min(100%, 1096px);
    gap: 16px;
    margin: 0 auto;
    padding: 1rem;

    .mods {
      grid-column: 1 / 2;
      grid-row: 1 / 3;

      .table {
        margin-bottom: 1rem;
        overflow: hidden;
        max-height: 32rem;
      }

      h3,
      p {
        text-align: center;
      }

      h4 {
        margin: 0;
        color: var(--color-contrast);
      }

      .search-bar {
        width: 100%;
        padding: var(--gap-sm);
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        border-radius: var(--radius-md);
        border: 1px solid var(--landing-border-color);
        background: linear-gradient(0deg, #3b3f55 0%, #3b3f55 100%), rgba(59, 63, 85, 0.15);
        margin-bottom: 0.5rem;
        white-space: nowrap;
        font-size: var(--font-size-sm);

        .mini-input {
          display: flex;
          flex-direction: row;
          align-items: center;
          gap: 0.5rem;
          padding: var(--gap-sm) var(--gap-md);
          border-radius: var(--radius-sm);
          background-color: #1e202f;
          flex-grow: 1;
          max-width: 12rem;
        }

        h4 {
          font-weight: normal;
          margin-left: 0.5rem;
        }
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 2fr 1fr 3.75rem;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          flex-direction: column;
          justify-content: center;
          padding: var(--gap-sm) 0;
          font-size: var(--font-size-sm);

          .name {
            color: var(--color-contrast);
          }

          .description {
            font-size: var(--font-size-xs);
          }

          &.last {
            align-items: flex-end;
          }

          &.check {
            align-items: center;
            flex-direction: row;
          }
        }
      }

      .header {
        .cell {
          color: var(--color-base);
        }
      }
    }

    .playing {
      grid-column: 1 / 2;
      grid-row: 4 / 5;
      position: relative;
      min-height: 23rem;

      .minecraft {
        position: absolute;
        bottom: 6rem;
        right: var(--gap-xl);
        height: 13rem;
        mask-image: none;
      }

      .launcher {
        position: absolute;
        z-index: 12;
        top: var(--gap-xl);
        left: var(--gap-xl);
        height: 13rem;
        mask-image: none;
      }

      .text {
        position: absolute;
        left: 50%;
        bottom: 0;
        transform: translateX(-50%);
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        color: var(--color-contrast);
        text-align: center;
        padding: var(--gap-xl);
        mask-image: none;
        z-index: 10;
        width: 100%;
      }
    }

    .sharing {
      grid-column: 2 / 3;
      grid-row: 2 / 3;

      .table {
        height: 11rem;
        overflow: hidden;
        margin-bottom: 1rem;
      }

      h3,
      p {
        text-align: center;
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 1fr 2fr;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          align-items: center;
          padding: var(--gap-sm) 0;

          .description {
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
            overflow: hidden;
          }
        }
      }

      .header {
        display: flex;
        flex-direction: row;
        gap: 0.25rem;
        padding: var(--gap-sm) var(--gap-sm);
      }

      .export-card {
        position: absolute;
        right: 1rem;
        top: 1.5rem;
        display: flex;
        flex-direction: row;
        align-items: center;
        padding: var(--gap-sm);
        gap: var(--gap-md);
        border-radius: var(--radius-md);
        background: linear-gradient(0deg, #3b3f55 0%, #3b3f55 100%), rgba(59, 63, 85, 0.15);
        border: 1px solid var(--landing-border-color);
        margin-bottom: 1rem;
        width: 20rem;
        font-size: var(--font-size-sm);

        img {
          width: 4.5rem;
          height: 4.5rem;
        }

        .info {
          width: 100%;
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          gap: 0.25rem;

          .exporting {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
            gap: 0.5rem;
            width: 100%;

            .tag {
              display: flex;
              flex-direction: row;
              align-items: center;
              gap: 0.25rem;
              border-radius: var(--radius-sm);
            }

            .small-button {
              display: flex;
              flex-direction: row;
              align-items: center;
              gap: 0.25rem;
              padding: var(--gap-xs) var(--gap-md);
              border-radius: var(--radius-lg);
              border: 1px solid var(--landing-border-color);
              font-size: var(--font-size-xs);
            }
          }

          .name {
            font-size: var(--font-size-md);
            color: var(--color-contrast);
            margin: 0;
          }
        }
      }
    }

    .performance {
      grid-column: 1 / 2;
      grid-row: 3 / 4;

      .title {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        margin-bottom: 0.5rem;

        h3 {
          margin: 0;
          font-size: var(--font-size-lg);
        }
      }

      h3,
      h4,
      p {
        text-align: center;
        margin: 0;
      }

      h4,
      h3 {
        color: var(--color-contrast);
        margin-top: var(--gap-lg) !important;
      }

      h4 {
        font-size: var(--font-size-md);
        font-weight: normal;
      }

      .table {
        height: 10rem;
        overflow: hidden;
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 2fr 1fr 1fr;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          align-items: center;
          padding: var(--gap-sm) 0;
          color: var(--color-gray);

          &.important {
            color: var(--color-contrast);
            font-weight: 500;
          }
        }
      }

      .header {
        font-size: var(--font-size-sm);
        .cell {
          color: var(--color-base);
        }
      }

      .icon-logo {
        width: 3rem;
        height: 3rem;
        padding: 0.5rem;
        border-radius: var(--radius-sm);
        background-color: var(--color-button-bg);

        &.modrinth {
          background-color: var(--color-brand) !important;
          border: 1px solid var(--color-accent-contrast) !important;

          .icon {
            width: 2rem;
            height: 2rem;
            z-index: 0;

            :deep(svg) {
              width: 2rem;
              height: 2rem;
            }

            :deep(path) {
              fill: var(--color-accent-contrast) !important;
            }
          }
        }
      }
    }

    .importing {
      position: relative;
      grid-column: 2 / 3;
      grid-row: 1 / 2;

      .icon-logo {
        position: absolute;
        top: 4rem;
        left: 50%;
        transform: translate(-50%, -50%);
        border-radius: 100%;
      }

      .outer-ring {
        position: absolute;
        top: 4rem;
        left: 50%;
        transform: translate(-50%, -50%);
      }

      .ring {
        position: relative;
        display: flex;
        justify-content: center;
        align-items: center;
        width: 100%;
        height: 100%;

        .base-ring {
          position: absolute;
          border-radius: 100%;
          top: 4rem;
          left: 50%;
          transform: translate(-50%, -50%);
          z-index: -1;
          border: 1px solid rgba(#a8b1ddbf, 0.25);
        }

        .first-ring {
          @extend .base-ring;
          width: 15rem;
          height: 15rem;
          background: radial-gradient(
            50% 50% at 50% 50%,
            rgba(5, 206, 69, 0.19) 0%,
            rgba(15, 19, 49, 0.25) 100%
          );
        }

        .second-ring {
          @extend .base-ring;
          width: 25rem;
          height: 25rem;
          opacity: 0.75;
          background: radial-gradient(
              50% 50% at 50% 50%,
              rgba(5, 206, 69, 0.19) 0%,
              rgba(15, 19, 49, 0.25) 100%
            ),
            radial-gradient(
              50% 50% at 50% 50%,
              rgba(44, 48, 79, 0.25) 0%,
              rgba(32, 35, 50, 0.19) 100%
            );
        }

        .third-ring {
          @extend .base-ring;
          width: 35rem;
          height: 35rem;
          opacity: 0.5;
          background: radial-gradient(
            50% 50% at 50% 50%,
            rgba(44, 48, 79, 0.25) 0%,
            rgba(32, 35, 50, 0.19) 100%
          );
        }
      }

      .inner-ring {
        position: relative;
      }

      .launcher-badge {
        position: absolute;
        background-color: var(--landing-transition-gradient-end);
        width: 3rem;
        height: 3rem;
        padding: 0.5rem;
        border-radius: 100%;
        z-index: 10;
        user-select: none;

        &:hover {
          cursor: default;
        }

        &.top-left {
          top: calc(15% - 3rem);
          left: calc(58% - 11rem);
        }

        &.top-right {
          top: calc(15% - 3rem);
          right: calc(58% - 11rem);
        }

        &.bottom-left {
          bottom: calc(41% + 3rem);
          left: calc(58% - 11rem);
        }

        &.bottom-right {
          bottom: calc(41% + 3rem);
          right: calc(58% - 11rem);
        }

        &.bottom-middle {
          bottom: calc(22% + 3rem);
          left: 50%;
          transform: translateX(-50%);
        }

        &.center {
          top: calc(50% - 3rem);
          left: 50%;
          transform: translate(-50%, -50%);
        }

        svg {
          width: 2rem;
          height: 2rem;
        }

        img {
          width: 2rem;
          height: 2rem;
        }
      }

      .text {
        position: absolute;
        text-align: center;
        left: 50%;
        bottom: 0;
        transform: translateX(-50%);
        width: 100%;
        padding: var(--gap-xl);
      }
    }

    .website {
      grid-column: 2 / 3;
      grid-row: 3 / 5;
      text-align: center;
      padding: 0 !important;

      position: relative;

      .projects-showcase {
        margin: calc(7rem + var(--gap-xl)) 0 var(--gap-xl);
        z-index: 3;
        text-align: left;

        .row {
          --gap: var(--gap-md);

          width: 100vw;
          gap: var(--gap);
          margin-bottom: var(--gap);
          display: flex;
          overflow: hidden;
          user-select: none;

          .row__content {
            flex-shrink: 0;
            display: flex;
            min-width: 100%;
            gap: var(--gap);
            transform: translateX(-15%);

            &.offset {
              transform: translateX(-130%);
            }
          }

          .project {
            position: relative;
            display: flex;

            cursor: pointer;
            padding: 1rem;
            gap: 1rem;
            border-radius: 1rem;
            border: 1px solid var(--landing-border-color);
            transition:
              background 0.5s ease-in-out,
              transform 0.05s ease-in-out;
            // Removed due to lag on mobile :(
            background: var(--landing-blob-gradient);

            img {
              height: 3rem;
            }

            .project-info {
              box-sizing: border-box;
            }

            .title {
              color: var(--landing-color-heading);
              max-width: 13.75rem;
              overflow: hidden;
              white-space: nowrap;
              text-overflow: ellipsis;
              margin: 0;
              font-weight: 600;
              font-size: 1.25rem;
              line-height: 110%;
              display: block;
            }

            .description {
              width: 13.75rem;

              display: -webkit-box;
              -webkit-line-clamp: 2;
              -webkit-box-orient: vertical;
              overflow: hidden;

              font-weight: 500;
              font-size: 0.875rem;
              line-height: 125%;
              margin: 0.25rem 0 0;
            }
          }
        }
      }

      .ellipsis {
        position: absolute;
        top: 2rem;
        left: 50%;
        height: 150%;
        z-index: -1;
        aspect-ratio: 1;
        border-radius: 100%;
        transform: translate(-50%, 2rem);
        background: var(--landing-blob-gradient);
        border: 1px solid var(--landing-border-color);
      }

      .icon-logo {
        position: absolute;
        top: 2rem;
        left: 50%;
        transform: translate(-50%, 0);
        z-index: 4;
      }

      p {
        padding: var(--gap-xl);
        padding-top: 0;
      }
    }

    .feature {
      padding: var(--gap-xl);
      z-index: 1;
      background: radial-gradient(
        50% 50% at 50% 50%,
        rgba(44, 48, 79, 0.35) 0%,
        rgba(32, 35, 50, 0.27) 100%
      );
      box-shadow:
        2px 2px 12px 0px rgba(0, 0, 0, 0.16),
        2px 2px 64px 0px rgba(57, 61, 94, 0.45) inset;
      backdrop-filter: blur(6px);
      -webkit-backdrop-filter: blur(6px);
      max-width: 540px;
      width: 100%;
      overflow: hidden;

      .additional-label {
        width: fit-content;
        padding: 0.5rem 0.75rem;
        margin-bottom: 0.5rem;
        background: var(--landing-blue-label-bg);
        color: var(--landing-blue-label);
        border-radius: 6px;
        font-weight: 700;
        font-size: 1rem;
      }

      h3,
      p {
        margin: 0;
      }

      h3 {
        font-size: var(--font-size-xl);
        color: var(--landing-color-heading);
        margin-bottom: 0.375rem;
      }

      p {
        color: var(--landing-color-subheading);
      }
    }
  }

  .feature-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--gap-lg);
    max-width: 1096px;
    margin: 0 auto;
    padding: calc(var(--gap-xl) * 2) 1rem;

    @media (max-width: 1024px) {
      grid-template-columns: repeat(1, 1fr);

      .point {
        text-align: center;

        .title {
          justify-content: center;
        }
      }
    }

    .point {
      display: flex;
      flex-direction: column;
      gap: var(--gap-md);
      padding: 1rem 0;

      svg {
        width: 1.5rem;
        height: 1.5rem;
      }

      .title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
      }

      h3 {
        font-size: var(--font-size-lg);
        font-weight: normal;
        color: var(--landing-color-heading);
        margin: 0;
      }

      p {
        color: var(--landing-color-subheading);
        margin: 0;
      }

      a {
        text-decoration: underline;
      }
    }
  }
}

.table {
  display: grid;
  border: 1px solid rgba(#a8b1ddbf, 0.25);
  gap: 0.25rem;
  overflow: hidden;
  font-size: var(--font-size-sm);
  background: rgba(59, 63, 85, 0.15);
  box-shadow: 2px 2px 12px 0px rgba(0, 0, 0, 0.16);

  .btn,
  button {
    &:hover {
      cursor: default !important;
    }
  }

  .first {
    border-top: none !important;
  }

  .row {
    &:not(.header) {
      border-top: 1px solid rgba(#a8b1ddbf, 0.25);
    }
  }
}

.row,
.header,
.table,
.project,
.export-card {
  user-select: none;

  &:hover {
    cursor: default;
  }
}

.footer {
  padding: var(--gap-xl);
  background: var(--color-accent-contrast);
  color: var(--color-contrast);
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: var(--gap-xl);
  justify-content: center;
  align-items: center;

  .section-badge {
    background-color: var(--color-brand-highlight);
    color: var(--color-brand);
    border-radius: var(--radius-sm);
    width: min-content;
    padding: var(--gap-lg) var(--gap-xl);
    white-space: nowrap;
  }

  .section-subheader {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--gap-sm);
    font-size: var(--font-size-lg);
    margin: 2rem 0;

    .section-subheader-title {
      font-size: var(--font-size-xl);
      margin: 0;
    }

    .section-subheader-description {
      color: var(--color-base);
      margin: 0;
    }
  }

  .download-section {
    display: grid;
    grid-template-columns: 1fr 1px 1fr 1px 1fr;
    height: 100%;
    gap: var(--gap-lg);
    max-width: 1096px;
    margin: 0 auto;

    @media (max-width: 1024px) {
      grid-template-columns: repeat(1, 1fr);
      max-width: 340px;

      .divider {
        display: none;
      }
    }

    .divider {
      height: 13rem;
      width: 1px;
      background: var(--landing-border-color);
      margin: 0;
    }

    .download-card {
      display: flex;
      flex-direction: column;
      gap: calc(var(--gap-lg) * 2);
      padding: calc(var(--gap-lg) * 2);
      height: min-content;

      .title {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        font-size: var(--font-size-2xl);
        gap: var(--gap-sm);
        border-radius: var(--radius-md) var(--radius-md) 0 0;
        color: var(--color-contrast);
      }

      .description {
        display: flex;
        flex-direction: column;
        align-items: center;
        border-top: none;
        font-size: var(--font-size-md);
        color: var(--color-brand);
        gap: var(--gap-sm);

        a {
          display: flex;
          align-items: flex-start;
          gap: var(--gap-sm);
          justify-content: center;

          &:hover {
            cursor: pointer;
          }
        }

        &.apple {
          align-items: flex-start;
        }
      }

      :deep(.animated-dropdown) {
        color: var(--color-brand);
        width: 16rem;
        white-space: nowrap;

        .selected {
          border: 1px solid var(--color-brand);
          background-color: var(--color-accent-contrast);
        }

        .options {
          border: 1px solid var(--color-brand);
          border-radius: 0 0 var(--radius-md) var(--radius-md);
        }

        .option {
          background-color: var(--color-accent-contrast);
        }

        .selected-option {
          background-color: var(--color-brand);
        }
      }
    }
  }

  .terms {
    margin: var(--gap-xl);
    font-size: var(--font-size-lg);
    color: var(--landing-color-subheading);
    text-align: center;
    line-height: 1.5;

    a {
      text-decoration: underline;
    }
  }
}

.icon-logo {
  background: var(--color-accent-contrast);
  width: 5rem;
  height: 5rem;
  padding: 0.75rem;
  border: 1px solid var(--landing-border-color);
  border-radius: var(--radius-lg);

  .icon {
    margin: auto;
    height: 3.5rem;

    :deep(svg) {
      width: 3.5rem;
      height: 3.5rem;
    }

    :deep(.rotate) {
      animation: none !important;
    }
  }
}

.gradient-border {
  position: relative;
  border-radius: var(--radius-lg);

  &:before {
    content: "";
    position: absolute;
    inset: 0;
    padding: 1px;
    z-index: -1;
    border-radius: 1rem;
    background: var(--landing-border-gradient);

    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
  }
}

.bottom-transition {
  position: absolute;
  bottom: 0;
  width: 100%;
  height: 30rem;
  background: linear-gradient(
    0deg,
    var(--landing-transition-gradient-end) 0%,
    var(--landing-transition-gradient-start) 100%
  );
}

@media screen and (max-width: 1024px) {
  .feature-grid {
    grid-template-columns: 1fr !important;
    grid-template-rows: repeat(8, 1fr) !important;
    gap: var(--gap-lg);
    margin: 0 auto;
    align-content: center;

    .feature {
      width: 100% !important;
      margin: 0 auto;
    }

    .mods {
      grid-row: 1 / 3 !important;
      grid-column: 1 / 2 !important;
    }

    .importing {
      grid-row: 3 / 4 !important;
      grid-column: 1 / 2 !important;
    }

    .sharing {
      grid-row: 4 / 5 !important;
      grid-column: 1 / 2 !important;
    }

    .performance {
      grid-row: 5 / 6 !important;
      grid-column: 1 / 2 !important;
    }

    .playing {
      grid-row: 6 / 7 !important;
      grid-column: 1 / 2 !important;
    }

    .website {
      grid-row: 7 / 9 !important;
      grid-column: 1 / 2 !important;
    }
  }

  .main-header {
    font-size: 4rem !important;
  }

  .subheader {
    font-size: 1.5rem !important;
  }

  .main-subheader {
    font-size: 1.25rem !important;
  }
}

@media screen and (max-width: 746px) {
  .main-header {
    font-size: 3rem !important;
  }

  .subheader {
    font-size: 1.25rem !important;
  }

  .main-subheader {
    font-size: 1.1rem !important;
  }

  .logo-banner {
    padding: 3rem 1rem 3.75rem 1rem;

    .overlay {
      bottom: 3.5rem;
    }
  }
}

.dark-icon {
  display: block;
}

.light-icon {
  display: none;
}

.light-mode {
  .footer,
  .features {
    background: #f8f7f8;
  }

  .bottom-transition {
    background: linear-gradient(rgba(#f8f7f8, 0) 0%, #f8f7f8 100%);
  }

  .dark-icon {
    display: none;
  }

  .light-icon {
    display: block;
  }

  .feature {
    background: radial-gradient(
      50% 50% at 50% 50%,
      rgba(255, 255, 255, 0.35) 0%,
      rgba(255, 255, 255, 0.27) 100%
    ) !important;
    box-shadow:
      2px 2px 64px 0px rgba(255, 255, 255, 0.45) inset,
      2px 2px 12px 0px rgba(0, 0, 0, 0.16) !important;
    border: none !important;
  }

  .gradient-border {
    &:before {
      background: var(--landing-border-gradient-light);
    }
  }

  .search-bar {
    background: var(--color-raised-bg) !important;
    border: 2px solid var(--color-brand) !important;

    .mini-input {
      background: var(--color-raised-bg) !important;
      border: 2px solid var(--color-bg);
    }
  }

  .ellipsis {
    opacity: 0.75;
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .landing-hero {
    background: url("https://cdn-raw.modrinth.com/app-landing/cube-light.png") no-repeat center 4rem;
    background-size: cover;
  }

  .base-ring {
    border: 1px solid rgba(#a8b1ddbf, 0.25) !important;
  }

  .first-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .second-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .third-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .table {
    background: white;
  }

  .project {
    background: rgba(255, 255, 255, 0.8) !important;
  }

  .export-card {
    background: white !important;
  }
}

.main-header-strong {
  font-weight: 600;
  background-color: #00bd3c;
  background-image: linear-gradient(180deg, #a7d0ff 0%, var(--color-brand) 60%);
  background-size: 100%;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  -moz-text-fill-color: transparent;
  color: transparent;
}

.logo-banner {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem 1rem 6.75rem 1rem;
  overflow: hidden;
  width: 100%;
  background-color: var(--color-accent-contrast);

  svg {
    z-index: 2;
    width: auto;
    height: 32rem;
  }

  .overlay {
    z-index: 3;
    position: absolute;
    bottom: 5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.5rem;
    white-space: nowrap;
  }
}
</style>
