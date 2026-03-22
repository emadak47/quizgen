document.addEventListener('DOMContentLoaded', function() {

  // Landing page: type option selection
  // The <label> wrapping handles radio toggle via browser default, but we
  // set checked explicitly as a safeguard (e.g. if markup is restructured).
  document.querySelectorAll('.type-option').forEach(function(opt) {
    opt.addEventListener('click', function() {
      opt.closest('.type-options').querySelectorAll('.type-option').forEach(function(o) {
        o.classList.remove('sel');
      });
      opt.classList.add('sel');
      var radio = opt.querySelector('input[type="radio"]');
      if (radio) radio.checked = true;
    });
  });

  // Question page: choice selection with flare re-trigger
  document.querySelectorAll('.q-choice').forEach(function(choice) {
    choice.addEventListener('click', function() {
      choice.closest('.q-choices').querySelectorAll('.q-choice').forEach(function(c) {
        c.classList.remove('sel');
      });
      choice.classList.add('sel');
      var radio = choice.querySelector('input[type="radio"]');
      if (radio) radio.checked = true;
      var frame = choice.querySelector('.q-choice-frame');
      frame.style.animation = 'none';
      frame.offsetHeight;
      frame.style.animation = '';
    });
  });

});
