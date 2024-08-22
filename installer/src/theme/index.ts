import type { App } from 'vue';
import BaseButton from '../components/BaseButton.vue';
import BaseInput from '../components/BaseInput.vue';
import BaseCheckBox from '../components/BaseCheckBox.vue';
import BaseProgress from '../components/BaseProgress.vue';
import BaseRadio from '../components/BaseRadio.vue';

export default {
  install(app: App) {
    app.component('base-button', BaseButton);
    app.component('base-input', BaseInput);
    app.component('base-check-box', BaseCheckBox);
    app.component('base-progress', BaseProgress);
    app.component('base-radio', BaseRadio);
  },
};
