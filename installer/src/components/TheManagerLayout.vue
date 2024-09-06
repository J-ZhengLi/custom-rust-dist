<template>
  <main absolute top="0" bottom="0" left="0" right="0">
    <RouterView
      absolute
      w="full"
      style="height: calc(100% - 2rem)"
    ></RouterView>
    <footer absolute bottom="0" right="2rem" c="regular">
      管理工具版本 0.0.0
    </footer>
  </main>
</template>
