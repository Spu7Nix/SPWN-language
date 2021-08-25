extract obj_props

type @array_runtime


let _array_runtime_ind = 0;

impl @array_runtime {
	new: (count: @number) {
		_array_runtime_ind++;

		let slf = @array_runtime::{
			indexer: ?g,
			reset_target: ?g,
			output_id: counter(2i),
			input_id: counter(?i),
			done_indicator: counter(?i),
			indexer_block: ?b,
			blocks: [?b for $ in 0..count],
			slots: [counter(?i) for $ in 0..count]
		};

		let slf._trigfuncs = {
			reset: !{
				slf.indexer.move_to(slf.reset_target, y_only=true, duration=0.1);
				wait(0.1)
				slf.indexer.move_to(slf.reset_target, x_only=true, duration=0.1);
			}
		}
		slf._trigfuncs.index = !{
				let ind1 = !{
					slf.indexer.move(20, 0, duration=0);
					//wait(0.4)
					slf.input_id--;

					if slf.input_id > 0 {
						ind1!
					} else {
						slf.indexer.move(0, -10, duration=0.1);
						wait(0.1)
						slf._trigfuncs.reset!;
					}
				};

				ind1!;
			}

		let base_x = _array_runtime_ind * 255;
		let base_y = 315;

		$.add(obj {
			OBJ_ID: 279,
			GROUPS: [slf.reset_target],
			X: base_x,
			Y: base_y
		});

		$.add(obj {
			OBJ_ID: 1816,
			ITEM: slf.indexer_block,
			GROUPS: [slf.indexer],
			X: base_x,
			Y: base_y,
			DYNAMIC_BLOCK: true
		});

		let i = 0;
		for b in slf.blocks {
			i++;
			$.add(obj {
				OBJ_ID: 1816,
				ITEM: b,
				X: base_x + (60*i),
				Y: base_y - 30
			});

			[[on(collision(slf.indexer_block, b))]] !{
				slf.output_id = slf.slots[i-1];
				slf.done_indicator++;
			}
		}

		return slf;
	},

	index: (self, deco: @macro, input: @number | @counter) {
		self.input_id = input;
		self.input_id++;
		self._trigfuncs.index!;

		[[on(count(self.done_indicator.item, 1))]] !{
			self.done_indicator--;
			deco(self.output_id);
		}
	}
}



let test = @array_runtime::new(5);


test.slots[3] = 3;

[[test.index(3)]] (out) {
	if out > 2 {
		hide_player();
	}
}
