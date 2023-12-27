#include "rive/animation/linear_animation_instance.hpp"
#include "rive/animation/state_machine_bool.hpp"
#include "rive/animation/state_machine_input.hpp"
#include "rive/animation/state_machine_input_instance.hpp"
#include "rive/animation/state_machine_instance.hpp"
#include "rive/animation/state_machine_number.hpp"
#include "rive/animation/state_machine_trigger.hpp"
#include "rive/custom_property_boolean.hpp"
#include "rive/custom_property_number.hpp"
#include "rive/custom_property_string.hpp"
#include "rive/event.hpp"
#include "rive/factory.hpp"
#include "rive/file.hpp"
#include "rive/math/path_types.hpp"
#include "rive/math/raw_path.hpp"
#include "rive/math/vec2d.hpp"
#include "rive/renderer.hpp"
#include "rive/text/text_value_run.hpp"

extern "C"
{
    using namespace rive;

    typedef struct RawRustBuffer RawRustBuffer;
    typedef struct RawRustPath RawRustPath;
    typedef struct RawRustPaint RawRustPaint;
    typedef struct RawRustGradient RawRustGradient;
    typedef struct RawRustImage RawRustImage;
    typedef struct RawRustFactory RawRustFactory;
    typedef struct RawRustRenderer RawRustRenderer;
    typedef struct RawRustString RawRustString;
    typedef struct RawRustBTreeMap RawRustBTreeMap;

    void rive_rs_allocate_string(const RawRustString* string, const char* data, size_t len);

    typedef struct RawString
    {
        const char* data;
        size_t len;
    } RawString;
    enum class PropertyTag : uint8_t
    {
        Bool,
        Number,
        String,
    };
    typedef union Property
    {
        bool boolean;
        float number;
        RawString string;
    } Property;

    void rive_rs_insert_property(const RawRustBTreeMap* properties,
                                 const char* key_data,
                                 size_t key_len,
                                 PropertyTag value_tag,
                                 Property value_payload);

    typedef struct RendererEntries
    {
        const RawRustBuffer* (*buffer_new)(RenderBufferType, RenderBufferFlags, size_t);
        void (*buffer_release)(const RawRustBuffer*);
        uint8_t* (*buffer_map)(const RawRustBuffer*);
        void (*buffer_unmap)(const RawRustBuffer*);
        const RawRustPath* (*path_default)();
        const RawRustPath* (*path_new)(RawPath::Iter*, size_t, FillRule);
        void (*path_release)(const RawRustPath*);
        void (*path_reset)(const RawRustPath*);
        void (*path_extend)(const RawRustPath*, const RawRustPath*, const float* transform);
        void (*path_set_fill_rule)(const RawRustPath*, FillRule);
        void (*path_move_to)(const RawRustPath*, float, float);
        void (*path_line_to)(const RawRustPath*, float, float);
        void (*path_cubic_to)(const RawRustPath*, float, float, float, float, float, float);
        void (*path_close)(const RawRustPath*);
        const RawRustPaint* (*paint_default)();
        void (*paint_release)(const RawRustPaint*);
        void (*paint_set_style)(const RawRustPaint*, RenderPaintStyle);
        void (*paint_set_color)(const RawRustPaint*, ColorInt);
        void (*paint_set_thickness)(const RawRustPaint*, float);
        void (*paint_set_join)(const RawRustPaint*, StrokeJoin);
        void (*paint_set_cap)(const RawRustPaint*, StrokeCap);
        void (*paint_set_blend_mode)(const RawRustPaint*, BlendMode);
        void (*paint_set_gradient)(const RawRustPaint*, const RawRustGradient*);
        void (*paint_invalidate_stroke)(const RawRustPaint*);
        const RawRustGradient* (*gradient_new_linear)(float,
                                                      float,
                                                      float,
                                                      float,
                                                      const ColorInt*,
                                                      const float*,
                                                      size_t);
        const RawRustGradient* (
            *gradient_new_radial)(float, float, float, const ColorInt*, const float*, size_t);
        void (*gradient_release)(const RawRustGradient*);
        const RawRustImage* (*image_decode)(const uint8_t*, size_t);
        void (*image_release)(const RawRustImage*);
        void (*renderer_state_push)(const RawRustRenderer*);
        void (*renderer_state_pop)(const RawRustRenderer*);
        void (*renderer_transform)(const RawRustRenderer*, const float*);
        void (*renderer_set_clip)(const RawRustRenderer*, const RawRustPath*);
        void (*renderer_draw_path)(const RawRustRenderer*, const RawRustPath*, const RawRustPaint*);
        void (*renderer_draw_image)(const RawRustRenderer*, const RawRustImage*, BlendMode, float);
        void (*renderer_draw_image_mesh)(const RawRustRenderer*,
                                         const RawRustImage*,
                                         const RawRustBuffer*,
                                         const RawRustBuffer*,
                                         const RawRustBuffer*,
                                         BlendMode,
                                         float);
    } RendererEntries;

    class RustBuffer : public lite_rtti_override<RenderBuffer, RustBuffer>
    {
    private:
        const RawRustBuffer* m_buffer;
        const RendererEntries* m_entries;

    public:
        RustBuffer(RenderBufferType type,
                   RenderBufferFlags flags,
                   size_t sizeInBytes,
                   const RendererEntries* entries) :
            lite_rtti_override(type, flags, sizeInBytes),
            m_buffer(entries->buffer_new(type, flags, sizeInBytes)),
            m_entries(entries)
        {}
        ~RustBuffer() override { m_entries->buffer_release(m_buffer); }

        const RawRustBuffer* buffer() const { return m_buffer; }

        void* onMap() override { return static_cast<void*>(m_entries->buffer_map(m_buffer)); }
        void onUnmap() override { m_entries->buffer_unmap(m_buffer); }
    };

    class RustShader : public lite_rtti_override<RenderShader, RustShader>
    {
    private:
        const RawRustGradient* m_gradient;
        const RendererEntries* m_entries;

    public:
        RustShader(const RawRustGradient* gradient, const RendererEntries* entries) :
            m_gradient(gradient), m_entries(entries)
        {}
        ~RustShader() override { m_entries->gradient_release(m_gradient); }

        const RawRustGradient* gradient() const { return m_gradient; }
    };

    class RustImage : public lite_rtti_override<RenderImage, RustImage>
    {
    private:
        const RawRustImage* m_image;
        const RendererEntries* m_entries;

    public:
        RustImage(const RawRustImage* image, const RendererEntries* entries) :
            m_image(image), m_entries(entries)
        {}
        ~RustImage() override { m_entries->image_release(m_image); }

        const RawRustImage* image() const { return m_image; }
    };

    class RustPath : public lite_rtti_override<RenderPath, RustPath>
    {
    private:
        const RawRustPath* m_path;
        const RendererEntries* m_entries;

    public:
        RustPath(const RawRustPath* path, const RendererEntries* entries) :
            m_path(path), m_entries(entries)
        {}
        ~RustPath() override { m_entries->path_release(m_path); }

        const RawRustPath* path() const { return m_path; }

        void rewind() override { m_entries->path_reset(m_path); }
        void addRenderPath(RenderPath* path, const Mat2D& transform) override
        {
            LITE_RTTI_CAST_OR_RETURN(rustPath, RustPath*, path);
            m_entries->path_extend(m_path, rustPath->m_path, transform.values());
        }
        void fillRule(FillRule value) override { m_entries->path_set_fill_rule(m_path, value); }
        void moveTo(float x, float y) override { m_entries->path_move_to(m_path, x, y); }
        void lineTo(float x, float y) override { m_entries->path_line_to(m_path, x, y); }
        void cubicTo(float ox, float oy, float ix, float iy, float x, float y) override
        {
            m_entries->path_cubic_to(m_path, ox, oy, ix, iy, x, y);
        }
        virtual void close() override { m_entries->path_close(m_path); }
    };

    class RustPaint : public lite_rtti_override<RenderPaint, RustPaint>
    {
    private:
        const RawRustPaint* m_paint;
        const RendererEntries* m_entries;

    public:
        RustPaint(const RawRustPaint* paint, const RendererEntries* entries) :
            m_paint(paint), m_entries(entries)
        {}
        ~RustPaint() override { m_entries->paint_release(m_paint); }

        const RawRustPaint* paint() const { return m_paint; }

        void style(RenderPaintStyle style) override { m_entries->paint_set_style(m_paint, style); }
        void color(unsigned int value) override { m_entries->paint_set_color(m_paint, value); }
        void thickness(float value) override { m_entries->paint_set_thickness(m_paint, value); }
        void join(StrokeJoin value) override { m_entries->paint_set_join(m_paint, value); }
        void cap(StrokeCap value) override { m_entries->paint_set_cap(m_paint, value); }
        void blendMode(BlendMode value) override
        {
            m_entries->paint_set_blend_mode(m_paint, value);
        }
        void shader(rcp<RenderShader> shader) override
        {
            auto rustShader = lite_rtti_cast<RustShader*>(shader.get());

            if (rustShader)
            {
                m_entries->paint_set_gradient(m_paint, rustShader->gradient());
            }
        }
        void invalidateStroke() override { m_entries->paint_invalidate_stroke(m_paint); }
    };

    class RustFactory : public Factory
    {
    private:
        const RendererEntries* m_entries;

    public:
        RustFactory(const RendererEntries* entries) : m_entries(entries) {}

        rcp<RenderBuffer> makeRenderBuffer(RenderBufferType type,
                                           RenderBufferFlags flags,
                                           size_t len_in_bytes) override
        {
            return rcp<RenderBuffer>(new RustBuffer(type, flags, len_in_bytes, m_entries));
        }

        rcp<RenderShader> makeLinearGradient(float sx,
                                             float sy,
                                             float ex,
                                             float ey,
                                             const ColorInt colors[],
                                             const float stops[],
                                             size_t count) override
        {
            const RawRustGradient* gradient =
                m_entries->gradient_new_linear(sx, sy, ex, ey, colors, stops, count);
            return rcp<RenderShader>(new RustShader(std::move(gradient), m_entries));
        }

        rcp<RenderShader> makeRadialGradient(float cx,
                                             float cy,
                                             float radius,
                                             const ColorInt colors[],
                                             const float stops[],
                                             size_t count) override
        {
            const RawRustGradient* gradient =
                m_entries->gradient_new_radial(cx, cy, radius, colors, stops, count);
            return rcp<RenderShader>(new RustShader(std::move(gradient), m_entries));
        }

        rcp<RenderPath> makeRenderPath(RawPath& path, FillRule fill_rule) override
        {
            auto iter = path.begin();
            return make_rcp<RustPath>(m_entries->path_new(&iter, path.verbs().size(), fill_rule),
                                      m_entries);
        }

        rcp<RenderPath> makeEmptyRenderPath() override
        {
            return make_rcp<RustPath>(m_entries->path_default(), m_entries);
        }

        rcp<RenderPaint> makeRenderPaint() override
        {
            return make_rcp<RustPaint>(m_entries->paint_default(), m_entries);
        }

        rcp<RenderImage> decodeImage(Span<const uint8_t> encoded) override
        {
            return make_rcp<RustImage>(m_entries->image_decode(encoded.data(), encoded.size()),
                                       m_entries);
        }
    };

    class RustRenderer : public Renderer
    {
    private:
        const RawRustRenderer* m_renderer;
        const RendererEntries* m_entries;

    public:
        RustRenderer(const RawRustRenderer* renderer, const RendererEntries* entries) :
            m_renderer(renderer), m_entries(entries)
        {}
        ~RustRenderer() override {}

        void save() override { m_entries->renderer_state_push(m_renderer); }
        void restore() override { m_entries->renderer_state_pop(m_renderer); }
        void transform(const Mat2D& transform) override
        {
            m_entries->renderer_transform(m_renderer, transform.values());
        }
        void clipPath(RenderPath* path) override
        {
            LITE_RTTI_CAST_OR_RETURN(rustPath, RustPath*, path);
            m_entries->renderer_set_clip(m_renderer, rustPath->path());
        }
        void drawPath(RenderPath* path, RenderPaint* paint) override
        {
            LITE_RTTI_CAST_OR_RETURN(rustPath, RustPath*, path);
            LITE_RTTI_CAST_OR_RETURN(rustPaint, RustPaint*, paint);
            m_entries->renderer_draw_path(m_renderer, rustPath->path(), rustPaint->paint());
        }
        void drawImage(const RenderImage* image, BlendMode blend_mode, float opacity) override
        {
            LITE_RTTI_CAST_OR_RETURN(rustImage, const RustImage*, image);
            m_entries->renderer_draw_image(m_renderer, rustImage->image(), blend_mode, opacity);
        }
        void drawImageMesh(const RenderImage* image,
                           rcp<RenderBuffer> vertices_f32,
                           rcp<RenderBuffer> uvCoords_f32,
                           rcp<RenderBuffer> indices_u16,
                           uint32_t,
                           uint32_t,
                           BlendMode blend_mode,
                           float opacity) override
        {
            LITE_RTTI_CAST_OR_RETURN(rustImage, const RustImage*, image);
            LITE_RTTI_CAST_OR_RETURN(rustVertices, RustBuffer*, vertices_f32.get());
            LITE_RTTI_CAST_OR_RETURN(rustUVCoords, RustBuffer*, uvCoords_f32.get());
            LITE_RTTI_CAST_OR_RETURN(rustIndices, RustBuffer*, indices_u16.get());
            m_entries->renderer_draw_image_mesh(m_renderer,
                                                rustImage->image(),
                                                rustVertices->buffer(),
                                                rustUVCoords->buffer(),
                                                rustIndices->buffer(),
                                                blend_mode,
                                                opacity);
        }
    };

    typedef struct Command
    {
        PathVerb verb;
        const Vec2D* points;
    } Command;

    enum class InputTag : uint8_t
    {
        Bool,
        Number,
        Trigger,
    };

    const File* rive_rs_file_new(const uint8_t* data,
                                 size_t len,
                                 const RendererEntries* entries,
                                 ImportResult* result,
                                 RustFactory** factory)
    {
        RustFactory* rust_factory = new RustFactory(entries);
        auto file = rive::File::import({data, len}, rust_factory, result);

        *factory = rust_factory;

        return static_cast<const File*>(file.release());
    }

    void rive_rs_file_release(const File* file, RustFactory* factory)
    {
        std::unique_ptr<File> val(std::move(const_cast<File*>(file)));
        delete factory;
    }

    void rive_rs_instantiate_artboard(const File* file,
                                      const size_t* index,
                                      ArtboardInstance** artboard_instance)
    {
        if (index)
        {
            if (*index < file->artboardCount())
            {
                *artboard_instance = file->artboardAt(*index).release();
            }
        }
        else
        {
            auto ptr = file->artboardDefault();
            if (ptr)
            {
                *artboard_instance = ptr.release();
            }
        }

        if (*artboard_instance)
        {
            (*artboard_instance)->advance(0.0f);
        }
    }

    void rive_rs_instantiate_artboard_by_name(const File* file,
                                              const char* data,
                                              size_t len,
                                              ArtboardInstance** artboard_instance)
    {
        *artboard_instance = file->artboardNamed({data, len}).release();

        if (*artboard_instance)
        {
            (*artboard_instance)->advance(0.0f);
        }
    }

    void rive_rs_artboard_instance_release(const ArtboardInstance* artboard_instance)
    {
        std::unique_ptr<ArtboardInstance> val(
            std::move(const_cast<ArtboardInstance*>(artboard_instance)));
    }

    size_t rive_rs_artboard_component_count(const ArtboardInstance* artboard_instance)
    {
        return artboard_instance->objects().size();
    }

    const Core* rive_rs_artboard_get_component(const ArtboardInstance* artboard_instance,
                                               size_t index)
    {
        return artboard_instance->objects()[index];
    }

    uint16_t rive_rs_component_type_id(const Core* component) { return component->coreType(); }

    void rive_rs_component_name(const Component* component, const char** data, size_t* len)
    {
        if (static_cast<const Core*>(component)->is<Component>())
        {
            *data = component->name().data();
            *len = component->name().size();
        }
        else
        {
            *len = 0;
        }
    }

    void rive_rs_text_value_run_get_text(const TextValueRun* text_value_run,
                                         const char** data,
                                         size_t* len)
    {
        *data = text_value_run->text().data();
        *len = text_value_run->text().size();
    }

    void rive_rs_text_value_run_set_text(TextValueRun* text_value_run, const char* data, size_t len)
    {
        text_value_run->text({data, len});
    }

    void rive_rs_instantiate_linear_animation(ArtboardInstance* artboard_instance,
                                              const size_t* index,
                                              LinearAnimationInstance** linear_animation)
    {
        if (index)
        {
            if (*index < (size_t)artboard_instance->animationCount())
            {
                *linear_animation = artboard_instance->animationAt(*index).release();
            }
        }
        else
        {
            auto ptr = artboard_instance->animationAt(0);
            if (ptr)
            {
                *linear_animation = ptr.release();
            }
        }
    }

    void rive_rs_instantiate_linear_animation_by_name(ArtboardInstance* artboard_instance,
                                                      const char* data,
                                                      size_t len,
                                                      LinearAnimationInstance** linear_animation)
    {
        *linear_animation = artboard_instance->animationNamed({data, len}).release();
    }

    float rive_rs_linear_animation_time(const LinearAnimationInstance* linear_animation)
    {
        return linear_animation->time();
    }

    void rive_rs_linear_animation_set_time(LinearAnimationInstance* linear_animation, float time)
    {
        linear_animation->time(time);
    }

    bool rive_rs_linear_animation_is_forwards(const LinearAnimationInstance* linear_animation)
    {
        return linear_animation->direction() == 1;
    }

    void rive_rs_linear_animation_set_is_forwards(LinearAnimationInstance* linear_animation,
                                                  bool is_forwards)
    {
        linear_animation->direction(is_forwards ? 1 : -1);
    }

    bool rive_rs_linear_animation_advance(LinearAnimationInstance* linear_animation, float elapsed)
    {
        return linear_animation->advance(elapsed);
    }

    void rive_rs_linear_animation_apply(const LinearAnimationInstance* linear_animation, float mix)
    {
        linear_animation->apply(mix);
    }

    bool rive_rs_linear_animation_did_loop(const LinearAnimationInstance* linear_animation)
    {
        return linear_animation->didLoop();
    }

    void rive_rs_linear_animation_set_loop(LinearAnimationInstance* linear_animation, Loop loop)
    {
        linear_animation->loopValue(static_cast<int>(loop));
    }

    bool rive_rs_linear_animation_is_done(const LinearAnimationInstance* linear_animation)
    {
        return !linear_animation->keepGoing();
    }

    void rive_rs_instantiate_state_machine(ArtboardInstance* artboard_instance,
                                           const size_t* index,
                                           StateMachineInstance** state_machine)
    {

        if (index)
        {
            if (*index < (size_t)artboard_instance->stateMachineCount())
            {
                *state_machine = artboard_instance->stateMachineAt(*index).release();
            }
        }
        else
        {
            auto ptr = artboard_instance->defaultStateMachine();
            if (ptr)
            {
                *state_machine = ptr.release();
            }
            else if (artboard_instance->stateMachineCount())
            {
                *state_machine = artboard_instance->stateMachineAt(0).release();
            }
        }
    }

    void rive_rs_instantiate_state_machine_by_name(ArtboardInstance* artboard_instance,
                                                   const char* data,
                                                   size_t len,
                                                   StateMachineInstance** state_machine)
    {
        *state_machine = artboard_instance->stateMachineNamed({data, len}).release();
    }

    void rive_rs_state_machine_get_event(const StateMachineInstance* state_machine_instance,
                                         size_t index,
                                         Event** event,
                                         float* delay)
    {
        auto event_report = state_machine_instance->reportedEventAt(index);
        *event = event_report.event();
        *delay = event_report.secondsDelay();
    }

    size_t rive_rs_state_machine_event_count(const StateMachineInstance* state_machine_instance)
    {
        return state_machine_instance->reportedEventCount();
    }

    void rive_rs_event_name(const Event* event, const RawRustString* string)
    {
        rive_rs_allocate_string(string, event->name().data(), event->name().size());
    }

    void rive_rs_event_properties(const Event* event, const RawRustBTreeMap* properties)
    {
        for (auto child : event->children())
        {
            switch (child->coreType())
            {
                case CustomPropertyBoolean::typeKey:
                {
                    auto boolean = static_cast<CustomPropertyBoolean*>(child);

                    Property property;
                    property.boolean = boolean->propertyValue();

                    rive_rs_insert_property(properties,
                                            child->name().data(),
                                            child->name().size(),
                                            PropertyTag::Bool,
                                            property);

                    break;
                }
                case CustomPropertyNumber::typeKey:
                {
                    auto number = static_cast<CustomPropertyNumber*>(child);

                    Property property;
                    property.number = number->propertyValue();

                    rive_rs_insert_property(properties,
                                            child->name().data(),
                                            child->name().size(),
                                            PropertyTag::Number,
                                            property);

                    break;
                }
                case CustomPropertyString::typeKey:
                {
                    auto string = static_cast<CustomPropertyString*>(child);

                    Property property;
                    property.string = {string->propertyValue().data(),
                                       string->propertyValue().size()};

                    rive_rs_insert_property(properties,
                                            child->name().data(),
                                            child->name().size(),
                                            PropertyTag::String,
                                            property);

                    break;
                }
            }
        }
    }

    void rive_rs_state_machine_get_input(const StateMachineInstance* state_machine_instance,
                                         size_t index,
                                         InputTag* input_tag,
                                         SMIInput** input)
    {
        *input = state_machine_instance->input(index);

        if ((*input)->input()->is<StateMachineBool>())
        {
            *input_tag = InputTag::Bool;
        }

        if ((*input)->input()->is<StateMachineNumber>())
        {
            *input_tag = InputTag::Number;
        }

        if ((*input)->input()->is<StateMachineTrigger>())
        {
            *input_tag = InputTag::Trigger;
        }
    }

    size_t rive_rs_state_machine_input_count(const StateMachineInstance* state_machine_instance)
    {
        return state_machine_instance->inputCount();
    }

    const SMIBool* rive_rs_state_machine_get_bool(
        const StateMachineInstance* state_machine_instance,
        const char* name,
        size_t len)
    {
        std::string n(name, len);

        return state_machine_instance->getBool({name, len});
    }

    const SMINumber* rive_rs_state_machine_get_number(
        const StateMachineInstance* state_machine_instance,
        const char* name,
        size_t len)
    {
        std::string n(name, len);

        return state_machine_instance->getNumber({name, len});
    }

    const SMITrigger* rive_rs_state_machine_get_trigger(
        const StateMachineInstance* state_machine_instance,
        const char* name,
        size_t len)
    {
        std::string n(name, len);

        return state_machine_instance->getTrigger({name, len});
    }

    void rive_rs_input_name(const SMIInput* input, const char** data, size_t* len)
    {
        *data = input->name().data();
        *len = input->name().size();
    }

    bool rive_rs_bool_get(const SMIBool* bool_) { return bool_->value(); }

    void rive_rs_bool_set(SMIBool* bool_, bool val) { bool_->value(val); }

    float rive_rs_number_get(const SMINumber* number) { return number->value(); }

    void rive_rs_number_set(SMINumber* number, float val) { number->value(val); }

    void rive_rs_trigger_fire(SMITrigger* trigger) { trigger->fire(); }

    void rive_rs_scene_release(const Scene* scene)
    {
        std::unique_ptr<Scene> val(std::move(const_cast<Scene*>(scene)));
    }

    Command rive_rs_commands_next(RawPath::Iter* commands)
    {
        auto tuple = **commands;
        ++*commands;
        return {std::get<0>(tuple), std::get<1>(tuple)};
    }

    float rive_rs_scene_width(const Scene* scene) { return scene->width(); }

    float rive_rs_scene_height(const Scene* scene) { return scene->height(); }

    Loop rive_rs_scene_loop(const Scene* scene) { return scene->loop(); }

    bool rive_rs_scene_is_translucent(const Scene* scene) { return scene->isTranslucent(); }

    float rive_rs_scene_duration(const Scene* scene) { return scene->durationSeconds(); }

    bool rive_rs_scene_advance_and_apply(Scene* scene, float elapsed)
    {
        return scene->advanceAndApply(elapsed);
    }

    void rive_rs_scene_draw(Scene* scene,
                            const RawRustRenderer* renderer,
                            const RendererEntries* entries)
    {
        RustRenderer rust_renderer(renderer, entries);
        scene->draw(&rust_renderer);
    }

    void rive_rs_scene_pointer_down(Scene* scene, float x, float y) { scene->pointerDown({x, y}); }

    void rive_rs_scene_pointer_move(Scene* scene, float x, float y) { scene->pointerMove({x, y}); }

    void rive_rs_scene_pointer_up(Scene* scene, float x, float y) { scene->pointerUp({x, y}); }

    void rive_rs_artboard_instance_transforms(const ArtboardInstance* artboard_instance,
                                              uint32_t width,
                                              uint32_t height,
                                              float* view_transform,
                                              float* inverse_view_transform)
    {
        auto view_transform_mat = rive::computeAlignment(rive::Fit::contain,
                                                         rive::Alignment::center,
                                                         rive::AABB(0, 0, width, height),
                                                         artboard_instance->bounds());
        auto inverse_view_transform_mat = view_transform_mat.invertOrIdentity();

        std::copy(view_transform_mat.values(), view_transform_mat.values() + 6, view_transform);
        std::copy(inverse_view_transform_mat.values(),
                  inverse_view_transform_mat.values() + 6,
                  inverse_view_transform);
    }
}
