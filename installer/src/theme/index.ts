import type { App } from 'vue';
import BaseButton from '../components/BaseButton.vue';
import BaseInput from '../components/BaseInput.vue';
import BaseCheckBox from '../components/BaseCheckBox.vue';
import BaseProgress from '../components/BaseProgress.vue';
import BaseRadio from '../components/BaseRadio.vue';
import BaseDialog from '../components/BaseDialog.vue';
import BaseTag from '@/components/BaseTag.vue';
import ScrollBox from '../components/ScrollBox.vue';

export default {
  install(app: App) {
    app.component('base-button', BaseButton);
    app.component('base-input', BaseInput);
    app.component('base-check-box', BaseCheckBox);
    app.component('base-progress', BaseProgress);
    app.component('base-radio', BaseRadio);
    app.component('base-dialog', BaseDialog);
    app.component('base-tag', BaseTag);
    app.component('scroll-box', ScrollBox);
  },
};
