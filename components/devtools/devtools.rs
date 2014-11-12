/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



use devtools_traits;
use devtools_traits::{DevtoolsControlChan, DevtoolsControlPort, NewGlobal, NodeInfo, GetRootNode};
use devtools_traits::{DevtoolScriptControlMsg, EvaluateJS, EvaluateJSReply, GetDocumentElement};
use devtools_traits::{GetChildren, GetLayout};


/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct DevtoolsTask {
    /// For providing instructions to an optional devtools server.
    devtools_chan: Option<DevtoolsControlChan>,
    /// For receiving commands from an optional devtools server. Will be ignored if
    /// no such server exists.
    devtools_port: DevtoolsControlPort
}

impl DevtoolsTask{

    fn handle_evaluate_js(&self, pipeline: PipelineId, eval: String, reply: Sender<EvaluateJSReply>) {
        let page = get_page(&*self.page.borrow(), pipeline);
        let frame = page.frame();
        let window = frame.as_ref().unwrap().window.root();
        let cx = window.get_cx();
        let rval = window.evaluate_js_with_result(eval.as_slice());

        reply.send(if rval.is_undefined() {
            devtools_traits::VoidValue
        } else if rval.is_boolean() {
            devtools_traits::BooleanValue(rval.to_boolean())
        } else if rval.is_double() {
            devtools_traits::NumberValue(FromJSValConvertible::from_jsval(cx, rval, ()).unwrap())
        } else if rval.is_string() {
            //FIXME: use jsstring_to_str when jsval grows to_jsstring
            devtools_traits::StringValue(FromJSValConvertible::from_jsval(cx, rval, conversions::Default).unwrap())
        } else {
            //FIXME: jsvals don't have an is_int32/is_number yet
            assert!(rval.is_object_or_null());
            fail!("object values unimplemented")
        });
    }

    fn handle_get_root_node(&self, pipeline: PipelineId, reply: Sender<NodeInfo>) {
        let page = get_page(&*self.page.borrow(), pipeline);
        let frame = page.frame();
        let document = frame.as_ref().unwrap().document.root();

        let node: JSRef<Node> = NodeCast::from_ref(*document);
        reply.send(node.summarize());
    }

    fn handle_get_document_element(&self, pipeline: PipelineId, reply: Sender<NodeInfo>) {
        let page = get_page(&*self.page.borrow(), pipeline);
        let frame = page.frame();
        let document = frame.as_ref().unwrap().document.root();
        let document_element = document.GetDocumentElement().root().unwrap();

        let node: JSRef<Node> = NodeCast::from_ref(*document_element);
        reply.send(node.summarize());
    }

    fn find_node_by_unique_id(&self, pipeline: PipelineId, node_id: String) -> Temporary<Node> {
        let page = get_page(&*self.page.borrow(), pipeline);
        let frame = page.frame();
        let document = frame.as_ref().unwrap().document.root();
        let node: JSRef<Node> = NodeCast::from_ref(*document);

        for candidate in node.traverse_preorder() {
            if candidate.get_unique_id().as_slice() == node_id.as_slice() {
                return Temporary::from_rooted(candidate);
            }
        }

        fail!("couldn't find node with unique id {:s}", node_id)
    }

    fn handle_get_children(&self, pipeline: PipelineId, node_id: String, reply: Sender<Vec<NodeInfo>>) {
        let parent = self.find_node_by_unique_id(pipeline, node_id).root();
        let children = parent.children().map(|child| child.summarize()).collect();
        reply.send(children);
    }

    fn handle_get_layout(&self, pipeline: PipelineId, node_id: String, reply: Sender<(f32, f32)>) {
        let node = self.find_node_by_unique_id(pipeline, node_id).root();
        let elem: JSRef<Element> = ElementCast::to_ref(*node).expect("should be getting layout of element");
        let rect = elem.GetBoundingClientRect().root();
        reply.send((rect.Width(), rect.Height()));
    }
}